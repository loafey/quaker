bold=$(tput bold)
normal=$(tput sgr0)
orange="\e[94m"

cd qwaks
folders=$(ls)
cd ..
mkdir -p assets/qwaks
for proj in ${folders}; do
    cargo build -p $proj --target wasm32-unknown-unknown --release
    echo -e "    ${bold}${orange}Compiled${normal} \"$proj\" QWAK file to: \"target/wasm32-unknown-unknown/release/$proj.wasm\"" 
    cp "target/wasm32-unknown-unknown/release/$proj.wasm" "assets/qwaks/$proj.wasm"
    echo -e "      ${bold}${orange}Copied${normal} \"$proj\" QWAK file to asset directory" 
done

cargo run --release