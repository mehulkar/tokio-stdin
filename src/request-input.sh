#!/usr/bin/env bash

echo -n "Do you like peanut butter?"
read -r answer

if [[ $answer == "y" ]]; then
    echo "Proceeding with peanut butter"
else
    echo "Skipping peanut butter"
fi
