#!/bin/bash

mkdir -p ~/.push/
mkdir -p ~/.push/bin/
touch ~/.push/.pushrc

echo "export "PATH=~/.push/:\$PATH" >> ~/.push/.pushrc