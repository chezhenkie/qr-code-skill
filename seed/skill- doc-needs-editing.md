make-qr.py          2287  generator, tested on a dummy string
phone-token.png      485  current phone token QR
phone-token.html     492  viewer that scales it to 70vmin
README.md            897  usage and dependency
The generator works offline and writes both files:
python .\make-qr.py "text" -o .\out.png --html --title "label"
python .\make-qr.py -f .\secret.txt -o .\secret.png --scale 12
Test produced a valid PNG and viewer page; the test folder is removed. The workspace copy in temp\qr is gone; repo is clean.
For the skill later: make-qr.py is the seed. The only dependency is segno (pure Python, installed user-level). A skill could either keep that pip step or vendor the few hundred lines of QR encoding; I would keep the dependency and document it, since QR encoders are easy to get subtly wrong.
One caution: phone-token.png is a live secret on the Desktop. Keep it until the phone is verified, then delete it. The old token tk_4ql7 is still valid until you revoke it, so the fallback stays intact.