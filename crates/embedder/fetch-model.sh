#!/bin/sh
# Fetches the weights of an embedding model at a pinned Hugging Face revision and checks every file
# against its sha256: the image build (deploy/api.Dockerfile, target embedder) and `make model`.
#   fetch-model.sh [multilingual-e5-small|rubert-tiny2] [destination, default models]
set -eu

model="${1:-multilingual-e5-small}"
dest="${2:-models}"

case "$model" in
multilingual-e5-small)
    repo=intfloat/multilingual-e5-small
    revision=614241f622f53c4eeff9890bdc4f31cfecc418b3
    sums="69137736cab8b8903a07fe8afaafdda25aac55415a12a55d1bffa9f581abf959  config.json
0b44a9d7b51c3c62626640cda0e2c2f70fdacdc25bbbd68038369d14ebdf4c39  tokenizer.json
1a55775f53449dac10a2bcbc312469fac40b96d53198c407081a831f81c98477  model.safetensors"
    ;;
rubert-tiny2)
    repo=cointegrated/rubert-tiny2
    revision=e8ed3b0c8bbf4fb6984c3de043bf7d2f4e5969ae
    sums="adee8b3e344bcb8379f44d0b3577c267d52881341e05d973c43e49974778dfff  config.json
45cc9f974145661db6bc020795839d1dc371adc19a9c78b910393209b4fe5efc  tokenizer.json
26ebb6db2a68593c54c74902d7a74f332da66297693f965cc9f1b0af4abf3894  model.safetensors"
    ;;
*)
    echo "unknown model: $model (multilingual-e5-small or rubert-tiny2)" >&2
    exit 1
    ;;
esac

if command -v sha256sum >/dev/null 2>&1; then
    check="sha256sum -c -"
else
    check="shasum -a 256 -c -"
fi

dir="$dest/$model"
mkdir -p "$dir"
cd "$dir"
if echo "$sums" | $check >/dev/null 2>&1; then
    echo "$model: already here and verified ($dir)"
    echo "$revision" >REVISION
    exit 0
fi
for file in config.json tokenizer.json model.safetensors; do
    echo "$model: $file"
    curl -fSL --retry 3 --retry-delay 2 -o "$file.part" "https://huggingface.co/$repo/resolve/$revision/$file"
    mv "$file.part" "$file"
done
echo "$sums" | $check
echo "$revision" >REVISION
echo "$model: revision $revision verified in $dir"
