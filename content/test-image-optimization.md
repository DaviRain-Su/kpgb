---
title: Test Image Optimization
author: KPGB Team
tags: [test, images, optimization]
category: feature-demo
excerpt: Testing the new image optimization feature that compresses images during build time.
---

# Image Optimization Test

This post demonstrates the new image optimization feature in KPGB. When you build your site, all images are automatically optimized to reduce file size while maintaining quality.

## Features

The image optimization system includes:

- **Automatic resizing**: Images larger than 1920x1080 are automatically resized
- **JPEG optimization**: JPEG images are compressed with 85% quality by default
- **PNG optimization**: PNG images are optimized for smaller file size
- **Preserves directory structure**: Images maintain their relative paths

## How It Works

During the site generation process (`kpgb generate`), the system:

1. Scans all image files in the output directory
2. Optimizes each image based on its format
3. Only replaces the original if the optimized version is smaller
4. Reports statistics about bytes saved

## Configuration

You can customize the optimization settings in the code:

```rust
ImageOptimizationConfig {
    max_width: 1920,
    max_height: 1080,
    jpeg_quality: 85,
    png_compression: 6,
}
```

This feature helps ensure your blog loads quickly for all visitors!