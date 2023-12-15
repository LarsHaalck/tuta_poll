#!/usr/bin/bash

echo "---------------"

sys_model=$(curl -s https://raw.githubusercontent.com/tutao/tutanota/master/src/api/entities/sys/ModelInfo.ts | grep -Po "(?<=version: )[0-9]+")
tuta_model=$(curl -s https://raw.githubusercontent.com/tutao/tutanota/master/src/api/entities/tutanota/ModelInfo.ts | grep -Po "(?<=version: )[0-9]+")
echo "MODEL_VERSION"
echo "${sys_model}.${tuta_model}"

echo "---------------"

client_version=$(curl -s https://raw.githubusercontent.com/tutao/tutanota/master/package.json | grep -Po "(?<=\"version\": \")[0-9]+(\.[0-9]+)*")
echo "CLIENT_VERSION"
echo "${client_version}"

