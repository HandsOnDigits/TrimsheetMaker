# TrimsheetMaker

**TrimsheetMaker** is a 3D trimsheet creation and editing tool built with Rust and Bevy. It is designed to provide a dedicated workspace for preparing 3D meshes, editing UV layouts, managing atlas materials, and exporting optimized geometry.

The goal is to make working with trimsheets more convenient by bringing mesh inspection, UV editing, and atlas management together in a single application.

## Features

### 3D Mesh Editor
- Import and inspect 3D models.
- View imported objects and their individual mesh primitives.
- Navigate the 3D viewport using an orbit camera.
- Inspect mesh geometry, normals, tangents, and UV coordinates.
- Control object and primitive visibility.
- Select mesh primitives for editing.

### UV Editing
- Inspect UV layouts and UV islands.
- Select and manipulate UV islands.
- Transform UV coordinates directly within the editor.
- Apply UV transformations to individual mesh primitives.
- Preview UV changes in the 3D viewport.

### Trimsheet Atlas
- Create and manage atlas materials.
- Import textures into materials.
- Support for texture channels such as:
  - Base Color
  - Normal
  - Roughness
  - Metallic
  - Other supported material textures
- Load and preview EXR textures.
- Organize materials and their associated textures.

### Model Export
- Export models as GLB.
- Export geometry without embedding materials or textures.
- Combine compatible primitives belonging to the same original object.
- Preserve object transforms.
- Skip hidden geometry.
- Optimize mesh data during export.

### Editor Interface
- Integrated 3D mesh viewport and 2D atlas view.
- Material and scene management panels.
- Context-sensitive editing controls.
- Import dialogs.
- Loading overlays with progress feedback.
- Unsaved-change handling.

## Technology Stack

TrimsheetMaker is written in Rust and uses Bevy as its rendering and application framework.

| Technology | Purpose |
|---|---|
| Rust | Core programming language |
| Bevy 0.19 | Rendering and application framework |
| bevy_egui | Editor interface |
| EXR | High-dynamic-range texture support |
| glTF | 3D model handling |
| Rayon | Parallel processing |
| image | Image processing and texture loading |
| Serde | Serialization and data handling |
| RON | Project data serialization |

## Requirements

- Rust toolchain supporting the Rust 2024 edition.
- Cargo.
- A graphics card and graphics drivers compatible with Bevy.

## Getting Started

Clone the repository:

```bash
git clone <repository-url>
cd trimsheet_maker
```

Build the project:

```bash
cargo build --release
```

Run the editor:

```bash
cargo run --release
```

For development, run without release optimizations:

```bash
cargo run
```

## Project Structure

The application is organized into separate modules responsible for editor functionality.

| Module | Responsibility |
|---|---|
| `data` | Atlas, material, and project data |
| `mesh` | Mesh scene management and editing |
| `import` | Model and material importing |
| `export` | Atlas and model exporting |
| `history` | Undo and redo functionality |
| `ui` | Editor interface |
| `assets` | Texture and asset handling |
| `linking` | Relationships between editor data and mesh objects |
| `native_dialog` | Native file dialogs and file operations |

## Design Goals

TrimsheetMaker is being developed around a few central principles:

- **Non-destructive editing:** UV transformations should remain editable without unnecessarily modifying the underlying mesh geometry.
- **Geometry-focused workflows:** Keep geometry processing separate from material and texture data.
- **Performance:** Take advantage of Rust and parallel processing where appropriate.
- **Predictable exports:** Preserve original object relationships and transforms while avoiding unnecessary geometry and material duplication.
- **Integrated workflow:** Keep mesh inspection, UV editing, and atlas management within one application.

## Development Status

TrimsheetMaker is under active development.

Features and workflows are being implemented incrementally, and the application is not yet considered a stable release.
