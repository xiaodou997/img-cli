# Third-party PSD fixtures

The files below are redistributed for YuTool's M3 engine conformance corpus.

## psd-tools

Repository: `psd-tools/psd-tools`

Pinned source commit:

```text
f1256273ffb9b39b9efa19d486c86f594c831f42
```

License: MIT. The complete upstream license notice is included at `licenses/psd-tools-MIT.txt`.

| YuTool path | Upstream path | Notes |
| --- | --- | --- |
| `upstream/psd-tools/2layers.psd` | `tests/psd_files/2layers.psd` | copied unchanged |
| `upstream/psd-tools/2layers.psb` | `tests/psd_files/2layers.psb` | copied unchanged |
| `upstream/psd-tools/group.psd` | `tests/psd_files/group.psd` | copied unchanged |
| `upstream/psd-tools/type-layer.psd` | `tests/psd_files/layers-minimal/type-layer.psd` | copied unchanged |
| `upstream/psd-tools/masks-2.psd` | `tests/psd_files/masks/2.psd` | copied unchanged |
| `derived/duplicate-layer-names.psd` | `tests/psd_files/2layers.psd` | derived by YuTool; Unicode `luni` names changed to `X`, Pascal names preserved |

The copied and derived fixtures remain subject to the upstream MIT license notice.
