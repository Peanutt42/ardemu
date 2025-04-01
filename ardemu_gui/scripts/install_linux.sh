#!/bin/bash

project_root="$(realpath $(dirname $0)/../../)"

if [ "$project_root" = "" ]; then
	echo "failed to get absolute filepath of project root"
	exit 1
fi

cd $project_root

echo "Compiling..."

cargo b --release -p ardemu_gui

echo "Installing files..."

mkdir -p "$HOME/.local/bin"
cp "$project_root/target/release/ardemu_gui" "$HOME/.local/bin/ardemu"
mkdir -p "$HOME/.local/ardemu.app/icons/hicolor/512x512/apps"
cp "$project_root/ardemu_gui/assets/icon.png" "$HOME/.local/ardemu.app/icons/hicolor/512x512/apps/ardemu.png"
mkdir -p "$HOME/.local/share/applications"
cp "$project_root/ardemu_gui/assets/ardemu.desktop" "$HOME/.local/share/applications/ardemu.desktop"
sed -i "s|\$HOME|$HOME|g" "$HOME/.local/share/applications/ardemu.desktop" # replaces '$HOME' with '/home/user'

echo "Finished!"