#!/bin/bash

for f in *.mscgen; do
  mscgen -T png -o ${f%.mscgen}.png -i ${f}
done
