# MeshCore Patches

This directory contains patches for the MeshCore firmware submodule.

## meshcore-simulator-changes.patch

This patch modifies the MeshCore firmware to enable compilation as a shared library for use with the simulator. The changes include:

- Build system modifications for DLL/shared library output
- Simulation-specific adaptations for timing and hardware abstraction

### Applying the Patch

From the repository root:

```bash
cd MeshCore
git apply ../patches/meshcore-simulator-changes.patch
```

### Reverting the Patch

```bash
cd MeshCore
git checkout .
```

### Base Version

This patch is based on upstream MeshCore release:

- **Tag:** `companion-v1.17.1`
- **Commit:** `d92964352441e53b93e8667b802e04f6e072b39e`
- **Upstream:** <https://github.com/meshcore-dev/MeshCore.git>

The submodule points to our fork (<https://github.com/Brent-A/MeshCore.git>) which has these changes already applied at commit `2c802d93` (upstream release merged into the simulator branch).

### Notes

- If you update the MeshCore submodule to a newer upstream version, the patch may need to be regenerated.
- The simulator's build process (`mcsim-firmware` crate) automatically applies necessary build configurations, so manual patching is typically not required for normal use.
