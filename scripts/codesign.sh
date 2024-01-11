# This script is used to code sign, notarize, and verify a binary for distribution on macOS.
# It is specific to macOS and won't work on other operating systems.

# Path to the binary that needs to be signed and notarized
BINARY_PATH="target/release/checkmark"

# Code sign the binary
codesign --force -s "$MACOS_APPLE_DEVELOPER_ID" -v $BINARY_PATH --deep --strict --options=runtime --timestamp

# Create a zip file containing the signed binary
zip -r checkmark.zip $BINARY_PATH

# Code sign zip file
codesign --force -s "$MACOS_APPLE_DEVELOPER_ID" -v checkmark.zip --deep --strict --options=runtime --timestamp

# Notarize the zip file
xcrun notarytool submit checkmark.zip --keychain-profile "APPLE_SIGN_PROFILE" --wait

# Staple the notarization ticket to the zip file
# This allows the notarization to be verified offline
xcrun stapler staple checkmark.zip

# Verify the signature and notarization of the zip file
spctl -a -vvv -t install checkmark.zip
codesign -dv --verbose=4 checkmark.zip
