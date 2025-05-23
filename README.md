# view-img

Simple image viewing CLI.

img-view leverages terminal graphics protocols to display images in the terminal.

## Usage

Simply provide the path to the image you want to display as a positional argument. Depending on the quality of the image, it may take many seconds to load and render. You'll see a throbber to indicate that it's processing. At any time, you can press any key to exit the app.

The option `--[r]esize` can be used to specify how the image should be resized. `crop` will crop the image if the terminal is too small to display it in full, and make no change to the image's size. `fit` will shrink the image if the terminal is too small. `scale` will do the same, **but will also grow the image to fill the terminal** if the terminal is larger than the image. All three resizing modes will maintain the image's original aspect ratio. The default mode is `fit`.
