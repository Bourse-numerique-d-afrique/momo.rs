#!/bin/bash

set -e

# Read the current version from Cargo.toml
current_version=$(jq -r '.package.version' Cargo.toml)

# Increment the version (assuming semantic versioning)
new_version=$(echo $current_version | awk -F. '{$NF = $NF + 1;} 1' OFS=.)

# Update Cargo.toml with the new version
jq --arg new_version "$new_version" '.package.version = $new_version' Cargo.toml > Cargo.toml.tmp
mv Cargo.toml.tmp Cargo.toml

# Commit the changes
git add Cargo.toml
git commit -m "Bump version to $new_version"

# Tag the commit
git tag $new_version

# Push changes and tags to the repository
git push origin master  # Change 'main' to your branch name if it's different
git push origin $new_version
