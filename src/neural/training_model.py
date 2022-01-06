import tensorflow as tf
from tensorflow.keras import backend, layers, models

from neural.training_data import NN_TOTAL_INPUT_SIZE_PER_POS, NN_TOTAL_PLANES_PER_POS, NN_TOTAL_OUTPUT_SIZE_PER_POS


class TransposeChannelsLastLayer(layers.Layer):
    def __init__(self):
        super(TransposeChannelsLastLayer, self).__init__(trainable=False, name='transpose_channels_last')

    # def build(self, input_shape: TensorShape):
    #     pass

    def call(self, inputs, *args, **kwargs):
        return tf.transpose(inputs, perm=[0, 2, 3, 1], name=self.name)


class TrainingModel:
    @staticmethod
    def get_uncompiled_model():
        main_input = layers.Input(shape=NN_TOTAL_INPUT_SIZE_PER_POS, dtype='float32', name='main_input')
        reshape = layers.Reshape((NN_TOTAL_PLANES_PER_POS, 8, 8))(main_input)
        move_channels_last = TransposeChannelsLastLayer()(reshape)

        conv1 = layers.Conv2D(256, (3, 3), padding='same', activation='relu')(move_channels_last)
        # pooling1 = layers.MaxPooling2D((2, 2), padding='same', data_format='channels_first')(conv1)
        # dropout1 = layers.Dropout(0.2)(conv1)

        conv2 = layers.Conv2D(128, (3, 3), padding='same', activation='relu')(conv1)
        pooling2 = layers.MaxPooling2D((2, 2), padding='same')(conv2)
        dropout2 = layers.Dropout(0.2)(pooling2)

        conv3 = layers.Conv2D(256, (2, 2), padding='same', activation='relu')(dropout2)
        pooling3 = layers.MaxPooling2D((2, 2), padding='same')(conv3)
        dropout3 = layers.Dropout(0.2)(pooling3)

        flatten = layers.Flatten()(dropout3)
        dense1 = layers.Dense(NN_TOTAL_OUTPUT_SIZE_PER_POS >> 1)(flatten)
        main_output = layers.Dense(NN_TOTAL_OUTPUT_SIZE_PER_POS, name='output_raw')(dense1)

        main_output_mask = layers.Input(shape=(NN_TOTAL_OUTPUT_SIZE_PER_POS,), dtype='float32', name='output_mask')
        multiply = layers.Multiply()([main_output, main_output_mask])
        final_output = layers.Softmax(name='movement_output')(multiply)

        win_probability_output = layers.Dense(1, name='win_probability', activation='sigmoid')(main_output)

        model = models.Model([main_input, main_output_mask], [final_output, win_probability_output])

        print(model.summary())
        return model

    @staticmethod
    def get_compiled_model() -> models.Model:
        model = TrainingModel.get_uncompiled_model()
        model.compile(optimizer='adam',
                      loss={
                          'movement_output': tf.keras.losses.CategoricalCrossentropy(),
                          'win_probability': tf.keras.losses.MeanAbsoluteError()
                          # 'win_probability': tf.keras.losses.MeanSquaredError()
                      },
                      metrics=['accuracy'])
        return model