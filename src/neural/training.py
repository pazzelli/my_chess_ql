import argparse
import tensorflow as tf
import matplotlib.pyplot as plt

from training_data import EPOCHS_PER_BATCH
from neural.training_data import TrainingData
from neural.training_model import TrainingModel


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

        epochs_range = range(EPOCHS_PER_BATCH)

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
    def run_training(path):
        # print(tf.config.list_physical_devices())

        model = TrainingModel.get_compiled_model()

        for X_train_main_input, X_train_output_mask, X_train_output_target, X_train_win_result, X_val_main_input, \
            X_val_output_mask, X_val_output_target, X_val_win_result in TrainingData.get_next_encoded_positions(path):

            history = model.fit(
                x = {'main_input': X_train_main_input, 'output_mask': X_train_output_mask},
                y = {'movement_output': X_train_output_target, 'win_probability': X_train_win_result},
                # batch_size=512,
                epochs=EPOCHS_PER_BATCH,
                # shuffle=False,
                validation_data=(
                    {'main_input': X_val_main_input, 'output_mask': X_val_output_mask},
                    {'movement_output': X_val_output_target, 'win_probability': X_val_win_result}
                )
            )

            Training.visualize_training_results(history)
            break

    @staticmethod
    def main():
        parser = argparse.ArgumentParser(description='Train a neural network to play chess!')
        parser.add_argument('pgn_path', type=str,
                            help='a path containing Portable Game Notation (PGN) files to use for training')
        args = parser.parse_args()

        Training.run_training(args.pgn_path)


if __name__ == "__main__":
    Training.main()
