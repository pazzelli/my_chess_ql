import os

import my_chess_ql
import tensorflow as tf
from training_constants import *


class TrainingData:
    @staticmethod
    def get_next_pgn_file(path) -> str:
        for root, dirs, files in os.walk(path):
            for file in files:
                if file.endswith(b'pgn'):
                    yield os.path.join(root, file).decode('utf-8')

    @staticmethod
    def get_next_position(path) -> ([float], [float], [float], float, bool, bool):
        for file_path in TrainingData.get_next_pgn_file(path):
            # noinspection PyUnresolvedReferences
            pgn = my_chess_ql.NeuralTrainer(file_path)
            while True:
                try:
                    nn_data = pgn.__next__()
                    yield nn_data
                except StopIteration:
                    break
        return None

    @staticmethod
    def get_datasets(path: str) -> (tf.data.Dataset, tf.data.Dataset):
        ds_positions = tf.data.Dataset.from_generator(
            TrainingData.get_next_position,
            args=[path],
            output_types=(tf.float32, tf.float32, tf.float32, tf.float32, tf.bool, tf.bool),
            output_shapes=((NN_TOTAL_INPUT_SIZE_PER_POS,), (NN_TOTAL_OUTPUT_SIZE_PER_POS,), (NN_TOTAL_OUTPUT_SIZE_PER_POS,), (), (), ())
        )

        # Shuffle the raw dataset being generated from Rust
        # ds_positions = ds_positions.shuffle(buffer_size=SHUFFLE_BUFFER_SIZE, seed=12, reshuffle_each_iteration=False)

        # For every WINDOW_SIZE samples, split across train / test sets using the TRAIN_TEST_SPLIT percentage
        WINDOW_SIZE = 10
        split = int(TRAIN_TEST_SPLIT * WINDOW_SIZE)
        ds_train = ds_positions.shuffle(buffer_size=SHUFFLE_BUFFER_SIZE, seed=12, reshuffle_each_iteration=False) \
            .window(split, WINDOW_SIZE).flat_map(lambda *ds: ds[0] if len(ds) == 1 else tf.data.Dataset.zip(ds))
        ds_validation = ds_positions.skip(split).window(WINDOW_SIZE - split, WINDOW_SIZE).flat_map(lambda *ds: ds[0] if len(ds) == 1 else tf.data.Dataset.zip(ds))

        # Calculate batch sizes for train/test data using TRAIN_TEST_SPLIT
        # AUTOTUNE = tf.data.AUTOTUNE
        train_batch_size = int(BATCH_SIZE * TRAIN_TEST_SPLIT)
        val_batch_size = BATCH_SIZE - train_batch_size

        # Enable caching and prefetch for better performance for subsequent epochs
        # return ds_train.cache().prefetch(buffer_size=AUTOTUNE).batch(train_batch_size), \
        #        ds_validation.cache().prefetch(buffer_size=AUTOTUNE).batch(val_batch_size)
        # ds_train = ds_train.prefetch(buffer_size=AUTOTUNE).batch(train_batch_size)
        # ds_validation = ds_validation.prefetch(buffer_size=AUTOTUNE).batch(val_batch_size)
        ds_train = ds_train.batch(train_batch_size)
        ds_validation = ds_validation.batch(val_batch_size)

        x = ds_train.map(lambda main_input, output_mask, t, t2, t3, t4: {INPUT_MAIN_LAYER_NAME: main_input, OUTPUT_MASK_LAYER_NAME: output_mask})
        y = ds_train.map(lambda t, t2, output_target, win_result, t3, t4: {OUTPUT_MOVEMENTS_LAYER_NAME: output_target, OUTPUT_WIN_PROBABILITY_LAYER_NAME: win_result})

        x_val = ds_validation.map(lambda main_input, output_mask, t, t2, t3, t4: {INPUT_MAIN_LAYER_NAME: main_input, OUTPUT_MASK_LAYER_NAME: output_mask})
        y_val = ds_validation.map(lambda t, t2, output_target, win_result, t3, t4: {OUTPUT_MOVEMENTS_LAYER_NAME: output_target, OUTPUT_WIN_PROBABILITY_LAYER_NAME: win_result})

        return x.cache(), y.cache(), x_val, y_val

