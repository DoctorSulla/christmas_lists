#!/usr/bin/bash
folder_name="christmaslist.xyz"
app_name="christmas_lists"
rm -rf $folder_name
mkdir $folder_name
cargo build --release
cp -r assets $folder_name
cp -r target/release/$app_name $folder_name
tar -czvf $app_name.tar.gz $folder_name
rm -rf $folder_name
scp $app_name.tar.gz hroot:/srv/http/
rm $app_name.tar.gz
