vagrant up
vagrant ssh -c 'sudo -i -- <<EOF
cargo install cargo-test-junit
cd /vagrant
cargo test-junit --name results.xml
EOF'
vagrant destroy -f