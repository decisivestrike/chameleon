#!/bin/bash

set -e

for i in {1..10}; do
    busctl --user call \
        org.freedesktop.Notifications \
        /org/freedesktop/Notifications \
        org.freedesktop.Notifications \
        Notify \
        susssasa\{sv\}i \
        "test" 0 "dialog-information" "Test $i" "Some body" 0 0 5000

    sleep 0.3
done