# `artushak-web-assets`

This is a simple web asset manager. It can manage an asset manifest and generate new file version when a source file or asset manifest is changed. It can be useful in web to allow far future cache expiry headers.

For example, you may have SASS file and a correct manifest. If this file is updated and you run `pack` function, your file will be processed by filter and copied to output directory with new name.

## Asset manifest format

Asset manifest is a JSON dictionary with fields:

* `public_assets` is list of names of assets that should be copied to output directory, each list item is string
* `assets` is a list of asset definitions, each list item is a dictionary with keys:
    * `output_base_path` (optional), value is a prefix of output path (string)
    * `extension`, value is file extension (string)
    * `source`, value is file source data, a dictionary with either of keys:
        * `File` (if asset is loaded from a source file), value is a file path
        * `Filtered` (if asset is generated by filter), value is a dictionary with keys:
            * `filter_name`, value is a filter name (using filter registry)
            * `input_names`, value is a list of input asset names
            * `options`, value is dictionary with arbitary string keys with values passed to filter as options, possible values can be:
                * `"Flag"` is flag option
                * `{"String": "STRING"}` is string option (place value instead of `STRING`)
                * `{"StringVec": ["STRING1", "STRING2"]}` is string list option (place values instead of `STRING1`, `STRING`, etc)
                * `{"Bool": false}` is false boolean option
                * `{"Bool": true}` is true boolean option

### Example

```json
{
    "assets": {
        "style_sass": {
            "extension": "sass",
            "source": {
                "File": "style.sass"
            }
        },
        "style_css": {
            "extension": "css",
            "source": {
                "Filtered": {
                    "filter_name": "SASS",
                    "input_names": [
                        "style_sass"
                    ],
                    "options": {
                        "mode": {
                            "String": "Compressed"
                        }
                    }
                }
            }
        },
        "icon1_svg": {
            "output_base_path": "images",
            "extension": "svg",
            "source": {
                "File": "icon1.svg"
            }
        }
    },
    "public_assets": [
        "style_css",
        "icon1_svg"
    ]
}
```

## Asset filters

Asset filters are implementations of `AssetFilter` trait. They take list of input file paths and file path output. For example, asset filter can compile SASS, minify file content and so on. Filters also take dictionary of options from manifest.

## Misc

TODO
