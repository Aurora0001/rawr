echo "Building documentation..."
cargo doc --no-deps
rm -r ../doc
cp -r ./target/doc ../doc
git checkout gh-pages
rm -r ./doc
cp -r ../doc ./
git add .
git commit -m "Update documentation."
git push origin gh-pages
git checkout master
