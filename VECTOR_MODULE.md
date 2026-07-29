# Digital Product Factory (DPF)

Desktop app for product creation, publishing, and AI-generated marketing assets.

## Logo Generator & Vector Generator

Two new modules for creating AI-generated SVG graphics.

### Logo Generator (🎨 Logo Generator tab)

| Feature | Details |
|---|---|
| **Styles** | Minimal, Modern, Vintage, Playful, Corporate, Tech, HandDrawn |
| **Input** | Brand name, tagline, style, colors, icon description |
| **Output** | SVG icon + typography + combined full logo |
| **Favicon** | Auto-generates 16/32/48/192/512 PNGs + ICO + webmanifest |
| **Export** | SVG file, favicon package |
| **License** | Team tier |

### Vector Generator (📐 Vector Generator tab)

| Feature | Details |
|---|---|
| **Categories** | Icon, Illustration, Badge, Pattern, Decorative, Infographic, UI Element |
| **Input** | Name, category, prompt, style, colors |
| **Output** | SVG with viewBox + palette |
| **Export** | SVG or PNG (512px) |
| **License** | Team tier |

### Architecture

```
LLMRouter → SVG JSON → usvg parse → resvg render → preview / export
```

### Database

- `logos` table — brand_name, style, palette, icon_svg, typography_svg, full_svg, favicon_package
- `vector_assets` table — category, prompt, svg_content, palette, view_box
