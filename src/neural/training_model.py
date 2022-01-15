import tensorflow as tf
from tensorflow.keras import layers, models

from training_data import NN_TOTAL_INPUT_SIZE_PER_POS, NN_TOTAL_PLANES_PER_POS, NN_TOTAL_OUTPUT_SIZE_PER_POS, TOP_K_OUTPUTS


class TopKLayer(layers.Layer):
    def __init__(self, k: int, name='top_k', **kwargs):
        super(TopKLayer, self).__init__(name=name, **kwargs)
        self.k = k

    def compute_output_shape(self, input_shape):
        output_shape = input_shape
        output_shape[-1] = output_shape[-1] * 2
        return output_shape

    # def build(self, input_shape):
    #     super(TopKLayer, self).build(input_shape)

    # def call(self, inputs, training, *args, **kwargs):
    def call(self, inputs, training=False, mask=None):
        result = tf.math.top_k(input=inputs, k=self.k)
        # return zip(result.values, result.indices)
        return tf.concat([tf.cast(result.indices, tf.float32), result.values], -1)

    def get_config(self):
        return {"k": self.k}


class TransposeChannelsLastLayer(layers.Layer):
    def __init__(self):
        super(TransposeChannelsLastLayer, self).__init__(name='transpose_channels_last')

    # def build(self, input_shape: TensorShape):
    #     pass

    # def call(self, inputs, training, *args, **kwargs):
    def call(self, inputs, training=False, mask=None):
        return tf.transpose(inputs, perm=[0, 2, 3, 1])

    # def compute_mask(self, inputs, mask=None):
    #     # Just pass the received mask from previous layer, to the next layer or
    #     # manipulate it if this layer changes the shape of the input
    #     return mask
    #
    # def get_config(self):
    #     return {}
    #     # return {"a": self.var.numpy()}

    # # There's actually no need to define `from_config` here, since returning
    # # `cls(**config)` is the default behavior.
    # @classmethod
    # def from_config(cls, config):
    #     return cls(**config)


class TrainingModel:
    @staticmethod
    def get_uncompiled_model():
        main_input = layers.Input(shape=NN_TOTAL_INPUT_SIZE_PER_POS, dtype='float32', name='main_input')
        reshape = layers.Reshape((NN_TOTAL_PLANES_PER_POS, 8, 8))(main_input)
        move_channels_last = TransposeChannelsLastLayer()(reshape)

        conv1 = layers.Conv2D(512, (4, 4), padding='same', activation='relu')(move_channels_last)
        # pooling1 = layers.MaxPooling2D((2, 2), padding='same', data_format='channels_first')(conv1)
        # dropout1 = layers.Dropout(0.2)(conv1)

        conv2 = layers.Conv2D(256, (3, 3), padding='same', activation='relu')(conv1)

        conv3 = layers.Conv2D(256, (2, 2), padding='same', activation='relu')(conv2)
        pooling3 = layers.MaxPooling2D((2, 2), padding='same')(conv3)
        dropout3 = layers.Dropout(0.2)(pooling3)

        conv3 = layers.Conv2D(512, (2, 2), padding='same', activation='relu')(dropout3)
        pooling3 = layers.MaxPooling2D((2, 2), padding='same')(conv3)
        dropout3 = layers.Dropout(0.2)(pooling3)

        flatten = layers.Flatten()(dropout3)
        dense1 = layers.Dense(2048)(flatten)
        main_output = layers.Dense(NN_TOTAL_OUTPUT_SIZE_PER_POS, name='output_raw')(dense1)

        main_output_mask = layers.Input(shape=(NN_TOTAL_OUTPUT_SIZE_PER_POS,), dtype='float32', name='output_mask')
        multiply = layers.Multiply()([main_output, main_output_mask])
        final_output = layers.Softmax(name='movement_output')(multiply)
        top_k_outputs = TopKLayer(k=TOP_K_OUTPUTS, name='top_k_outputs')(final_output)

        win_probability_output = layers.Dense(1, name='win_probability', activation='sigmoid')(main_output)

        model = models.Model([main_input, main_output_mask], [final_output, win_probability_output, top_k_outputs])
        return model

    @staticmethod
    def get_compiled_model() -> models.Model:
        model = TrainingModel.get_uncompiled_model()
        model.compile(optimizer='adam',
                      loss={
                          'movement_output': tf.keras.losses.CategoricalCrossentropy(),
                          'win_probability': tf.keras.losses.MeanAbsoluteError(),
                          # 'top_k_outputs': None,
                      },
                      metrics={
                          'movement_output': tf.keras.metrics.CategoricalAccuracy(),
                          'win_probability': tf.keras.metrics.MeanAbsoluteError(),
                      }
                      )
        return model
