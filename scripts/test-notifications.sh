#!/bin/bash

set -e

# Group 1: No replace (10 separate notifications)
for i in {1..10}; do
    notify-send --icon=dialog-information -h "string:desktop-entry:firefox" \
        "No Replace $i" "Separate notification #$i" -t 2000
    sleep 0.3
done

sleep 1

# Group 2: With replace (10 updates of one notification)
ID=$(notify-send -p --icon=dialog-information \
    "Replace Group" "First in replacement group" -t 12000)

for i in {2..10}; do
    notify-send -r "$ID" --icon=dialog-information \
        "Replace Group $i" "Replacement #$i" -t 4000
    sleep 0.3
done

echo "Done"