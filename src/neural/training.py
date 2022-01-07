import os.path
import datetime
import argparse

import keras
import tensorflow as tf
import matplotlib.pyplot as plt

from training_data import EPOCHS
from neural.training_data import TrainingData
from neural.training_model import TrainingModel, TransposeChannelsLastLayer


class Training():
    @staticmethod
    def visualize_training_results(history):
        movement_output_acc = history.history['movement_output_accuracy']
        win_probablity_acc = history.history['win_probability_accuracy']
        val_movement_output_acc = history.history['val_movement_output_accuracy']
        val_win_probability_acc = history.history['val_win_probability_accuracy']

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
            initial_learning_rate=0.02,
            decay_steps=1000,
            decay_rate=0.9
        )

        model.optimizer = tf.keras.optimizers.Adam(learning_rate=lr_schedule)
        # optimizer.learning_rate.assign(0.01)
        # print(optimizer.learning_rate)

    @staticmethod
    def run_training(pgn_path: str, model_path: str, tensorboard_log_dir="logs/fit"):
        # print(tf.config.list_physical_devices())

        if os.path.exists(os.path.join(model_path, 'saved_model.pb')):
            model = keras.models.load_model(model_path, custom_objects={'transpose_channels_last': TransposeChannelsLastLayer})
        else:
            print("WARNING: NEW MODEL WILL BE TRAINED - no previous model found at {}".format(model_path))
            model = TrainingModel.get_compiled_model()
        Training.adjust_modeL_learning_rate(model)
        print(model.summary())

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
                steps_per_epoch=10,
                validation_data=tf.data.Dataset.zip((x_val, y_val)),
                validation_steps=10,
                use_multiprocessing=True,
                workers=3,
                callbacks=[tensorboard_callback]
            )
        except (InterruptedError, KeyboardInterrupt):
            pass
        finally:
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
