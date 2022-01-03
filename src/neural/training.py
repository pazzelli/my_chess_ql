import os
import sys
import argparse
import numpy as np
import tensorflow as tf
import my_chess_ql


class Training:
    @staticmethod
    def get_next_pgn_file(path):
        for root, dirs, files in os.walk(path):
            for file in files:
                if file.endswith('pgn'):
                    yield os.path.join(root, file)

    @staticmethod
    def get_next_position(path):
        for file_path in Training.get_next_pgn_file(path):
            # noinspection PyUnresolvedReferences
            pgn = my_chess_ql.NeuralTrainer(file_path)
            while True:
                try:
                    next_pos = pgn.__next__()
                    yield next_pos
                except StopIteration:
                    break
        return None

    @staticmethod
    def run_training(path):
        np.set_printoptions(threshold=sys.maxsize)

        i = 0

        for nn_data in Training.get_next_position(path):
            if not nn_data: break

            (input_planes, output_planes, output_target, game_result, white_to_move, is_new_game) = nn_data
            # input_piece_planes = np.array(input_piece_planes)

            input_planes = np.array(input_planes).reshape((1, (8 * 12) + 7, 8, 8))
            # input_aux_planes = np.array(input_aux_planes, dtype=float).reshape((1, 7, 8, 8))

            # if white_to_move:
            #     input_planes = input_planes.

            if i == 0:
                print(input_planes)
                # print(np.around(input_planes, decimals=2))

                # # This magic line will normalize the fifty-move and move-count planes to be between [0, 1]
                # input_aux_planes[:, 1:3, :, :] /= 256.0

                # print(input_aux_planes)

                # print(tf.keras.utils.normalize(input_aux_planes, axis=4))

            # if i % 100 == 0:
            #     print("input_piece_planes: \n{}".format(input_piece_planes))
            #     print("input_aux_planes: \n{}".format(input_aux_planes))
            #     print("output_planes: \n{}".format(output_planes))
            #     print("output_target: \n{}".format(output_target))
            #     print("game_result: \n{}".format(game_result))

            i += 1
            if i >= 100: break

    # @staticmethod
    # def train():
    #     tf.keras.models

    @staticmethod
    def main():
        parser = argparse.ArgumentParser(description='Train a neural network to play chess!')
        parser.add_argument('pgn_path', type=str,
                            help='a path containing Portable Game Notation (PGN) files to use for training')
        args = parser.parse_args()

        Training.run_training(args.pgn_path)


if __name__ == "__main__":
    Training.main()
