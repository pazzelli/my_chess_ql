import sys
import my_chess_ql


class Training:
    @staticmethod
    def main(argv):
        # noinspection PyUnresolvedReferences
        pgn = my_chess_ql.NeuralTrainer("/Users/John/Documents/chessdata/bundesliga2000.pgn")
        while True:
            try:
                nn_data = pgn.__next__()
                (input_piece_planes, input_aux_planes, output_planes, output_target, game_result) = nn_data

                print("input_piece_planes: \n{}".format(input_piece_planes))
                print("input_aux_planes: \n{}".format(input_aux_planes))
                print("output_planes: \n{}".format(output_planes))
                print("output_target: \n{}".format(output_target))
                print("game_result: \n{}".format(game_result))
                # print(nn_data)
                break
            except StopIteration:
                break

        # for x in pgn:
        #     print(x)
        #     i += 1
        #     if i == 5:
        #         break
        pass


if __name__ == "__main__":
    Training.main(sys.argv)
