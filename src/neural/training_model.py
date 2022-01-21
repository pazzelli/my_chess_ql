import keras.regularizers
import tensorflow as tf
from tensorflow.keras import layers, models
from training_constants import *


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
        result_values_norm = result.values / (tf.norm(result.values, ord=1) + 0.000001)    # to prevent NaN in case all inputs are 0.0
        return tf.concat([tf.cast(result.indices, tf.float32), result_values_norm], -1)

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
    def create_resnet_layer_block(name: str, input_layer: layers.Layer, filter_count: int, filter_size: tuple) -> layers.Layer:
        input_layer_temp = input_layer

        y = layers.Conv2D(filter_count, filter_size, padding='same', name='{}_conv1'.format(name), kernel_regularizer=keras.regularizers.l2(REGULARIZATION_PARAMETER), bias_regularizer=keras.regularizers.l2(REGULARIZATION_PARAMETER))(input_layer)
        y = layers.BatchNormalization(name='{}_bn1'.format(name))(y)
        y = layers.LeakyReLU(name='{}_relu1'.format(name))(y)

        y = layers.Conv2D(filter_count, filter_size, padding='same', name='{}_conv2'.format(name), kernel_regularizer=keras.regularizers.l2(REGULARIZATION_PARAMETER), bias_regularizer=keras.regularizers.l2(REGULARIZATION_PARAMETER))(y)
        # if input_layer_temp.shape[-1] != y.shape[-1]:
        #     input_layer_temp = layers.Conv2D(y.shape[-1], kernel_size=(1, 1), padding='same', name='conv_inp1x1_{}'.format(name), kernel_regularizer=keras.regularizers.l2(REGULARIZATION_PARAMETER), bias_regularizer=keras.regularizers.l2(REGULARIZATION_PARAMETER))(input_layer_temp)

        y = layers.Add(name='{}_add'.format(name))([y, input_layer_temp])  # the actual Resnet, where the input 'skips' over the intermediate layers
        y = layers.BatchNormalization(name='{}_bn2'.format(name))(y)
        y = layers.LeakyReLU(name='{}_relu2'.format(name))(y)
        return y    # the final output layer

    @staticmethod
    def add_resnet_layer_blocks(name: str, input_layer: layers.Layer, filter_count: int, filter_size: tuple, block_count: int) -> layers.Layer:
        last_output = input_layer
        for i in range(block_count):
            last_output = TrainingModel.create_resnet_layer_block(name='{}_{}'.format(name, (i+1)), input_layer=last_output, filter_count=filter_count, filter_size=filter_size)

        return last_output

    @staticmethod
    def add_inception_layer(incep_layer_num: int, input_layer: layers.Layer, max_filter_count: int, block_count: int = 2, pooling_type: str = 'max', pooling_size=(2, 2)) -> layers.Layer:
        # 1x1 bypass layer
        conv1x1 = layers.Conv2D(max_filter_count, kernel_size=(1, 1), padding='same', name='i{}_conv1x1'.format(incep_layer_num), kernel_regularizer=keras.regularizers.l2(REGULARIZATION_PARAMETER), bias_regularizer=keras.regularizers.l2(REGULARIZATION_PARAMETER))(input_layer)

        # 2x2 layers
        channel_reduce = layers.Conv2D(max_filter_count, kernel_size=(1, 1), padding='same', name='i{}_conv2x2_1x1'.format(incep_layer_num), kernel_regularizer=keras.regularizers.l2(REGULARIZATION_PARAMETER), bias_regularizer=keras.regularizers.l2(REGULARIZATION_PARAMETER))(input_layer)
        conv2x2 = TrainingModel.add_resnet_layer_blocks(
            name='i{}_conv2x2'.format(incep_layer_num),
            input_layer=channel_reduce,
            filter_count=max_filter_count,
            filter_size=(2, 2),
            block_count=block_count,
        )

        # 3x3 layers
        channel_reduce = layers.Conv2D(max_filter_count, kernel_size=(1, 1), padding='same', name='i{}_conv3x3_1x1'.format(incep_layer_num), kernel_regularizer=keras.regularizers.l2(REGULARIZATION_PARAMETER), bias_regularizer=keras.regularizers.l2(REGULARIZATION_PARAMETER))(input_layer)
        conv3x3 = TrainingModel.add_resnet_layer_blocks(
            name='i{}_conv3x3'.format(incep_layer_num),
            input_layer=channel_reduce,
            filter_count=max_filter_count,
            filter_size=(3, 3),
            block_count=block_count,
        )

        # max_pool layers
        max_pool = layers.MaxPooling2D((2, 2), padding='same', strides=1, name='i{}_maxpool2x2'.format(incep_layer_num))(input_layer)
        max_pool1x1 = layers.Conv2D(max_filter_count, kernel_size=(1, 1), padding='same', name='i{}_maxpool2x2_conv1x1'.format(incep_layer_num), kernel_regularizer=keras.regularizers.l2(REGULARIZATION_PARAMETER), bias_regularizer=keras.regularizers.l2(REGULARIZATION_PARAMETER))(max_pool)

        # Channel concatenation
        concat = layers.Concatenate(name='i{}_concat'.format(incep_layer_num))([conv1x1, conv2x2, conv3x3, max_pool1x1])

        # Optional pooling layer
        if pooling_type == 'max':
            return layers.MaxPooling2D(pooling_size, padding='valid', name='i{}_maxpool'.format(incep_layer_num))(concat)
        elif pooling_type == 'avg':
            return layers.AveragePooling2D(pooling_size, padding='valid', name='i{}_avgpool'.format(incep_layer_num))(concat)
        elif pooling_type == '':
            return concat
        else:
            raise ValueError("Invalid pooling_type {}".format(pooling_type))


        # conv_layers = []
        # for i in range(1, 4):
        #     if i < input_layer.shape[1]:
        #         conv_layers.append(
        #             TrainingModel.add_resnet_layer_blocks(
        #                 name='{}x{}_{}'.format(i, i, incep_layer_num),
        #                 input_layer=input_layer,
        #                 filter_count=max_filter_count >> min(2, i - 2),
        #                 filter_size=(i, i),
        #                 block_count=block_count,
        #                 pooling_size=pooling_size,
        #                 pooling_type=pooling_type
        #             )
        #         )
        #
        # return layers.Concatenate(name='concat_{}'.format(incep_layer_num))(conv_layers)

    @staticmethod
    def get_uncompiled_model():
        main_input = layers.Input(shape=NN_TOTAL_INPUT_SIZE_PER_POS, dtype='float32', name=INPUT_MAIN_LAYER_NAME)
        reshape = layers.Reshape((NN_TOTAL_PLANES_PER_POS, 8, 8))(main_input)
        move_channels_last = TransposeChannelsLastLayer()(reshape)

        inception1 = TrainingModel.add_inception_layer(1, move_channels_last, 64, block_count=2, pooling_type='')
        inception2 = TrainingModel.add_inception_layer(2, inception1, 64, block_count=2, pooling_type='max')
        inception3 = TrainingModel.add_inception_layer(3, inception2, 64, block_count=2, pooling_type='max')

        flatten = layers.Flatten()(inception3)
        # # dense1 = layers.Dense(2048)(flatten)


        output_raw = layers.Dense(NN_TOTAL_OUTPUT_SIZE_PER_POS, name=OUTPUT_RAW_LAYER_NAME, kernel_regularizer=keras.regularizers.l2(REGULARIZATION_PARAMETER), bias_regularizer=keras.regularizers.l2(REGULARIZATION_PARAMETER))(flatten)

        movement_output = layers.Softmax(name=OUTPUT_MOVEMENTS_LAYER_NAME)(output_raw)

        main_output_mask = layers.Input(shape=(NN_TOTAL_OUTPUT_SIZE_PER_POS,), dtype='float32', name=OUTPUT_MASK_LAYER_NAME)
        multiply = layers.Multiply()([movement_output, main_output_mask])
        top_k_movements = TopKLayer(k=TOP_K_OUTPUTS, name=OUTPUT_TOP_K_MOVEMENTS_LAYER_NAME)(multiply)

        # win_probability_output = layers.Dense(1, name=OUTPUT_WIN_PROBABILITY_LAYER_NAME, activation='sigmoid')(output_raw)
        win_probability_output = layers.Dense(1, name=OUTPUT_WIN_PROBABILITY_LAYER_NAME, activation='sigmoid')(flatten)

        model = models.Model([main_input, main_output_mask], [movement_output, top_k_movements, win_probability_output])
        return model

    @staticmethod
    def get_compiled_model() -> models.Model:
        model = TrainingModel.get_uncompiled_model()
        model.compile(optimizer='adam',
                      loss={
                          # OUTPUT_RAW_LAYER_NAME: tf.keras.losses.CategoricalCrossentropy(),
                          # OUTPUT_RAW_LAYER_NAME: tf.keras.losses.MeanAbsoluteError(),
                          OUTPUT_MOVEMENTS_LAYER_NAME: tf.keras.losses.CategoricalCrossentropy(),
                          OUTPUT_WIN_PROBABILITY_LAYER_NAME: tf.keras.losses.MeanAbsoluteError(),
                      },
                      metrics={
                          # OUTPUT_RAW_LAYER_NAME: tf.keras.metrics.CategoricalAccuracy(),
                          OUTPUT_MOVEMENTS_LAYER_NAME: tf.keras.metrics.CategoricalAccuracy(),
                          OUTPUT_WIN_PROBABILITY_LAYER_NAME: tf.keras.metrics.MeanAbsoluteError(),
                      }
                      )
        return model
