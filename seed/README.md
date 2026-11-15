# qr

Offline QR generation. No network, no service, one Python dependency.

## Files

- `make-qr.py`: generator. Writes a PNG, optionally a viewer page.
- `phone-token.png`, `phone-token.html`: current ntfy phone token. Sensitive; delete after the phone is set up.

## Dependency

```powershell
python -m pip install --user segno
```

## Usage

```powershell
python .\make-qr.py "text to encode" -o .\out.png --html --title "label"
python .\make-qr.py -f .\secret.txt -o .\secret.png --scale 12
```

Output is an absolute PNG path on stdout, then the HTML path when `--html` is used. Defaults: error correction M, scale 10, border 4. The viewer page scales the image to 70% of the viewport; it never prints the encoded text.

## Notes

- The token QR encodes the raw token. The ntfy app has no QR import; scan with a reader that copies text.
- Treat any QR that encodes a token as a secret file.
