import argparse
import tensorflow as tf
import matplotlib.pyplot as plt

from training_data import EPOCHS
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
    def run_training(path):
        # print(tf.config.list_physical_devices())

        model = TrainingModel.get_compiled_model()

        ds_train, ds_val = TrainingData.get_datasets(path)

        x = ds_train.map(lambda main_input, output_mask, t, t2, t3, t4: {'main_input': main_input, 'output_mask': output_mask})
        y = ds_train.map(lambda t, t2, output_target, win_result, t3, t4: {'movement_output': output_target, 'win_probability': win_result})

        x_val = ds_val.map(lambda main_input, output_mask, t, t2, t3, t4: {'main_input': main_input, 'output_mask': output_mask})
        y_val = ds_val.map(lambda t, t2, output_target, win_result, t3, t4: {'movement_output': output_target, 'win_probability': win_result})

        history = model.fit(
            x=tf.data.Dataset.zip((x, y)),  # not sure why this additional zip is needed, but it is
            # batch_size=512,   # this isn't allowed here since the datasets themselves are already batched
            epochs=EPOCHS,
            # shuffle=False,    # not allowed here either since the datasets are also already shuffled
            validation_data=tf.data.Dataset.zip((x_val, y_val))
        )

        Training.visualize_training_results(history)

    @staticmethod
    def main():
        parser = argparse.ArgumentParser(description='Train a neural network to play chess!')
        parser.add_argument('pgn_path', type=str,
                            help='a path containing Portable Game Notation (PGN) files to use for training')
        args = parser.parse_args()

        Training.run_training(args.pgn_path)


if __name__ == "__main__":
    Training.main()
