# Alexandria

To install the server on Linux:
- Download the latest release: ``wget -O alexandria https://github.com/m1lkin/alexandria/releases/latest``
- Add the file to the folder with the file ``.env`` with the following variables: ``MONGODB_URI``, ``MONGO_USERNAME``, ``PASSWORD``, ``SECRET``, ``SERVER_URL``
- Make server executable: ``chmod -x alexandria``
- Check that you are configured MongoDB and create user in database "alexandria"
- Check that your firewall not blocking your address
- Finally, launch the server

## Manual install

If you want to compile from source, just install sources, move into directory and launch ``cargo build --release``. Check that you install Rust, Cargo and Clang.
