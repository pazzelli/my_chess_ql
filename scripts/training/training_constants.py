# Neural Net training parameters
# SHUFFLE_BUFFER_SIZE = 4000
SHUFFLE_BUFFER_SIZE = 2000
TRAIN_TEST_SPLIT = 0.9  # must contain a single decimal place only
# BATCH_SIZE = 20     # preferably in multiples of 10 so train/test split will produce expected results
BATCH_SIZE = 500     # preferably in multiples of 10 so train/test split will produce expected results
# BATCHES_PER_EPOCH = 500
BATCHES_PER_EPOCH = 200
EPOCHS = 10

INITIAL_LEARNING_RATE = 0.01
LEARNING_RATE_DECAY_RATE = 0.9
LEARNING_RATE_DECAY_BATCH_COUNT = 2000

REGULARIZATION_PARAMETER = 0.0001

TOP_K_OUTPUTS = 8

# Neural net structure parameters
NN_PIECE_PLANES = 12    # 6 planes for each side's pieces
NN_AUX_PLANES = 7   # 1x colour, 1x total move count, 2x P1 castling, 2x P2 castling, 1x fifty move count
NN_MOVE_HISTORY_PER_POS = 8     # Buffer the last 8 moves and show them to the NN
NN_TOTAL_PIECE_PLANES_PER_POS = NN_MOVE_HISTORY_PER_POS * NN_PIECE_PLANES
NN_TOTAL_PLANES_PER_POS = NN_TOTAL_PIECE_PLANES_PER_POS + NN_AUX_PLANES

NN_TOTAL_INPUT_SIZE_PER_POS = NN_TOTAL_PLANES_PER_POS << 6
NN_TOTAL_OUTPUT_SIZE_PER_POS = 1858

# NN model constants
INPUT_MAIN_LAYER_NAME = 'main_input'
OUTPUT_MASK_LAYER_NAME = 'output_mask'
OUTPUT_RAW_LAYER_NAME = 'output_raw'
OUTPUT_MOVEMENTS_LAYER_NAME = 'movement_output'
OUTPUT_TOP_K_MOVEMENTS_LAYER_NAME = 'top_k_outputs'
OUTPUT_WIN_PROBABILITY_LAYER_NAME = 'win_probability'