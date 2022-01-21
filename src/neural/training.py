import os.path
import datetime
import argparse

import tensorflow as tf
from tensorflow import keras
from keras import Model
import matplotlib.pyplot as plt

from training_constants import *
from training_data import TrainingData
from training_model import TrainingModel, TransposeChannelsLastLayer, TopKLayer


class Training:
    @staticmethod
    def load_model(model_path: str) -> Model:
        if os.path.exists(os.path.join(model_path, 'saved_model.pb')):
            model = keras.models.load_model(model_path, custom_objects={
                'transpose_channels_last': TransposeChannelsLastLayer,
                'top_k_outputs': TopKLayer,
            })
        else:
            print("WARNING: NEW MODEL WILL BE TRAINED - no previous model found at {}".format(model_path))
            model = TrainingModel.get_compiled_model()

        print(model.summary())
        return model

    @staticmethod
    def visualize_training_results(history):
        movement_output_acc = history.history['movement_output_categorical_accuracy']
        win_probablity_acc = history.history['win_probability_mean_absolute_error']
        val_movement_output_acc = history.history['val_movement_output_categorical_accuracy']
        val_win_probability_acc = history.history['val_win_probability_mean_absolute_error']

        loss = history.history['loss']
        val_loss = history.history['val_loss']
        movement_output_loss = history.history['movement_output_loss']
        win_probablity_loss = history.history['win_probability_loss']
        val_movement_output_loss = history.history['val_movement_output_loss']
        val_win_probability_loss = history.history['val_win_probability_loss']

        epochs_range = range(EPOCHS)

        plt.figure(figsize=(8, 8))
        plt.subplot(1, 2, 1)
        plt.plot(epochs_range, movement_output_acc, label='Training Movement Accuracy')
        plt.plot(epochs_range, val_movement_output_acc, label='Validation Movement Accuracy')
        plt.legend(loc='lower right')
        plt.title('Training and Validation Accuracy')

        plt.subplot(1, 2, 2)
        plt.plot(epochs_range, loss, label='Training Loss')
        plt.plot(epochs_range, val_loss, label='Validation Loss')
        plt.legend(loc='upper right')
        plt.title('Training and Validation Loss')
        plt.show()

    @staticmethod
    def adjust_modeL_learning_rate(model):
        lr_schedule = tf.keras.optimizers.schedules.ExponentialDecay(
            initial_learning_rate=INITIAL_LEARNING_RATE,
            decay_steps=LEARNING_RATE_DECAY_BATCH_COUNT,
            decay_rate=LEARNING_RATE_DECAY_RATE
        )

        model.optimizer = tf.keras.optimizers.Adam(learning_rate=lr_schedule)
        # optimizer.learning_rate.assign(0.01)
        # print(optimizer.learning_rate)

    @staticmethod
    def run_training(pgn_path: str, model_path: str, tensorboard_log_dir="../../logs/fit"):
        # print(tf.config.list_physical_devices())

        model = Training.load_model(model_path=model_path)
        Training.adjust_modeL_learning_rate(model)

        x, y, x_val, y_val = TrainingData.get_datasets(pgn_path)

        tensorboard_dir = os.path.join(tensorboard_log_dir, datetime.datetime.now().strftime("%Y%m%d-%H%M%S"))
        tensorboard_callback = tf.keras.callbacks.TensorBoard(log_dir=tensorboard_dir, histogram_freq=1)

        history = None
        try:
            history = model.fit(
                x=tf.data.Dataset.zip((x, y)),  # not sure why this additional zip is needed, but it is
                # batch_size=512,   # this isn't allowed here since the datasets themselves are already batched
                # shuffle=False,    # not allowed here either since the datasets are also already shuffled
                epochs=EPOCHS,
                steps_per_epoch=BATCHES_PER_EPOCH,
                validation_data=tf.data.Dataset.zip((x_val, y_val)),
                validation_steps=5,
                use_multiprocessing=True,
                workers=3,
                callbacks=[tensorboard_callback]
            )
        except (InterruptedError, KeyboardInterrupt):
            pass

        print("\nSaving model to {}".format(model_path))
        model.save(filepath=model_path, overwrite=True)

        if history:
            Training.visualize_training_results(history)

    @staticmethod
    def main():
        parser = argparse.ArgumentParser(description='Train a neural network to play chess!')
        parser.add_argument('pgn_path', type=str,
                            help='a path containing Portable Game Notation (PGN) files to use for training')
        parser.add_argument('model_path', type=str,
                            help='a path to the folder of an existing model file (to resume from) or where to save existing results')
        args = parser.parse_args()

        Training.run_training(args.pgn_path, args.model_path)


if __name__ == "__main__":
    Training.main()
