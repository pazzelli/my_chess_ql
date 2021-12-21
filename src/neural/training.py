import sys
import my_chess_ql


class Training:
    @staticmethod
    def main(argv):
        # print(my_chess_ql.get_positions_from_pgn_file("/Users/John/Documents/chessdata/bundesliga2000.pgn"))
        # print(my_chess_ql.get_positions_from_pgn_file("/Users/John/Documents/chessdata/bundesliga2000.pgn"))
        pgn = my_chess_ql.NeuralTrainer("/Users/John/Documents/chessdata/bundesliga2000.pgn")
        i = 0
        while True:
            try:
                line = str(pgn.__next__())
                print(line)
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
