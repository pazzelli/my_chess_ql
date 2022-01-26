source ~/.bash_profile

# ## These have already been run on the remote machine
# xcode-select --install
# /bin/bash -c “$(curl -fsSL https://github.com/conda-forge/miniforge/releases/latest/download/Mambaforge-MacOSX-arm64.sh)"
# conda config --set auto_activate_base false
# conda create --name mlp python=3.8
# conda init bash   # only needs to be run once to modify the bash shell for next login

conda activate mlp
conda install -c apple tensorflow-deps
pip install tensorflow-macos tensorflow-metal
