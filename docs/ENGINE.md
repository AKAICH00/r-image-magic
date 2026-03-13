# R-Image-Magic Generation Engine

The generation engine is the core of R-Image-Magic, responsible for transforming 2D designs into photorealistic 3D-looking product mockups using advanced image processing algorithms.

## 1. Compositing Pipeline

The pipeline follows these stages to generate a mockup:

1.  **Fetching Design**: Downloads the design from the provided URL. Supports PNG and JPG.
2.  **Background Removal**: Automatically removes white/near-white backgrounds from design images using an edge-aware luminance thresholding algorithm.
3.  **Resizing**: Scales the design based on the `PlacementSpec` to match the print area dimensions of the template.
4.  **Displacement Mapping**: If enabled for the template, the design is distorted to follow fabric wrinkles and folds.
5.  **Blending**: Composites the design onto the base image using specified blend modes (Normal, Multiply, Screen, Overlay).
6.  **Encoding**: Returns the final result as a base64 encoded PNG or uploads it to a storage provider.

## 2. Displacement Mapping Algorithm

True displacement mapping is what sets R-Image-Magic apart. Unlike simple overlays, displacement mapping moves pixels of the design image to follow the physical topology of the fabric.

### How it works:
- A **Displacement Map** is a grayscale image where:
    - `128 (Gray)`: No displacement.
    - `0 (Black)`: Maximum negative displacement (left/up).
    - `255 (White)`: Maximum positive displacement (right/down).
- The engine uses **Bilinear Interpolation** for smooth pixel sampling, preventing aliasing during distortion.
- The `displacement_strength` parameter controls how aggressively pixels are shifted.

## 3. High-Performance Parallelism

To handle thousands of concurrent requests, the engine utilizes several performance optimizations:

- **Rayon Integration**: Image processing tasks (displacement, blending, background removal) are parallelized across all available CPU cores using the Rayon library.
- **Zero-Copy Buffers**: Minimizes memory allocations during the compositing process.
- **Pre-loaded Templates**: All template assets (base images, displacement maps, metadata) are loaded into memory at startup to eliminate disk I/O during generation requests.

## 4. Blend Modes

The engine supports several standard blending algorithms:

- **Normal**: Simple alpha-over blending.
- **Multiply**: Multiplies design colors with template colors. Essential for dark designs on light fabrics where fabric shadows should show through.
- **Screen**: Opposite of multiply, useful for light designs on dark fabrics.
- **Overlay**: Combination of multiply and screen. Preserves high-contrast details of both layers.
