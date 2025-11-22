# MSSCS Client - Modern Design System

## 🎨 Design Philosophy

The MSSCS client features a **flat AMOLED aesthetic** with incredible modern design principles:

- **Pure Black AMOLED** (#000000) background for energy efficiency and visual depth
- **Neon accent colors** with glow effects for visual hierarchy
- **Glassmorphism** elements with backdrop blur for depth perception
- **Smooth animations** and micro-interactions for delightful UX
- **Modern iconography** using clean SVG icons instead of emojis
- **Gradient accents** throughout for visual interest

## 🎯 Color System

### Background Colors
```css
--color-bg-primary: #000000      /* Pure AMOLED black */
--color-bg-secondary: #0a0a0a    /* Subtle elevation */
--color-bg-tertiary: #121212     /* Card backgrounds */
--color-bg-elevated: #1a1a1a     /* Hover states */
```

### Neon Accent Colors
```css
--color-accent-primary: #00ff88    /* Neon green - primary actions */
--color-accent-secondary: #00ccff  /* Cyan - secondary elements */
--color-accent-tertiary: #ff00ff   /* Magenta - special highlights */
--color-accent-warning: #ffaa00    /* Amber - warnings */
--color-accent-danger: #ff3366     /* Red - destructive actions */
```

### Gradients
```css
--gradient-primary: linear-gradient(135deg, #00ff88 0%, #00ccff 100%)
--gradient-secondary: linear-gradient(135deg, #ff00ff 0%, #00ccff 100%)
--gradient-danger: linear-gradient(135deg, #ff3366 0%, #ff6b9d 100%)
```

### Glow Effects
```css
--glow-primary: 0 0 20px rgba(0, 255, 136, 0.3)
--glow-secondary: 0 0 20px rgba(0, 204, 255, 0.3)
--glow-tertiary: 0 0 20px rgba(255, 0, 255, 0.3)
```

## 📐 Spacing System

```css
--spacing-xs: 4px
--spacing-sm: 8px
--spacing-md: 16px
--spacing-lg: 24px
--spacing-xl: 32px
```

## 🔲 Border Radius

```css
--radius-sm: 6px    /* Small elements */
--radius-md: 12px   /* Cards, buttons */
--radius-lg: 16px   /* Large cards */
--radius-xl: 24px   /* Modals */
--radius-full: 9999px /* Pills, toggles */
```

## ✨ Key Design Features

### 1. Glassmorphism
All cards and elevated surfaces use glassmorphism:
```css
.glass {
  background: rgba(255, 255, 255, 0.03);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.05);
}
```

### 2. Gradient Text
Titles use gradient text for visual impact:
```css
.gradient-text {
  background: var(--gradient-primary);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}
```

### 3. Glow Effects
Interactive elements have subtle glow on hover:
```css
.glow-effect {
  box-shadow: var(--glow-primary);
  animation: glow 2s ease-in-out infinite;
}
```

### 4. Smooth Transitions
All interactions use smooth cubic-bezier transitions:
```css
--transition-fast: 150ms cubic-bezier(0.4, 0, 0.2, 1)
--transition-base: 250ms cubic-bezier(0.4, 0, 0.2, 1)
--transition-slow: 350ms cubic-bezier(0.4, 0, 0.2, 1)
```

## 🎭 Component Patterns

### Primary Button
```vue
<button class="btn-primary">
  <svg>...</svg>
  <span>Action</span>
</button>
```
- Gradient background
- Glow effect on hover
- Lift animation (-2px translateY)
- SVG icon + text

### Secondary Button
```vue
<button class="btn-secondary">
  <svg>...</svg>
  <span>Action</span>
</button>
```
- Glassmorphism background
- Subtle border
- Hover state with increased opacity

### Card Component
```vue
<div class="card glass">
  <!-- Content -->
</div>
```
- Glassmorphism effect
- Hover lift animation
- Subtle border
- Rounded corners (--radius-lg)

### Status Indicator
```vue
<div class="status-indicator online">
  <div class="status-pulse"></div>
</div>
```
- Circular indicator
- Glow effect for online status
- Pulsing animation
- Color-coded states

## 🎬 Animations

### Fade In
```css
@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
```

### Slide In
```css
@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateX(-20px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}
```

### Pulse
```css
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}
```

### Glow
```css
@keyframes glow {
  0%, 100% {
    box-shadow: var(--glow-primary);
  }
  50% {
    box-shadow: 0 0 30px rgba(0, 255, 136, 0.5);
  }
}
```

## 📱 Responsive Design

The design is optimized for desktop but scales gracefully:
- Grid layouts use `auto-fill` with `minmax()`
- Flexible spacing with CSS variables
- Fluid typography
- Touch-friendly hit areas (minimum 40px)

## 🎨 Typography

### Font Family
```css
font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
```

### Font Weights
- 300: Light (subtle text)
- 400: Regular (body text)
- 500: Medium (labels)
- 600: Semibold (buttons, nav)
- 700: Bold (section titles)
- 800: Extra Bold (page titles)

### Text Hierarchy
```css
--color-text-primary: #ffffff    /* Main content */
--color-text-secondary: #a0a0a0  /* Supporting text */
--color-text-tertiary: #666666   /* Muted text */
--color-text-muted: #404040      /* Disabled text */
```

## 🎯 Accessibility

- High contrast ratios (WCAG AAA compliant)
- Focus states on all interactive elements
- Semantic HTML structure
- ARIA labels where needed
- Keyboard navigation support
- Reduced motion support (respects prefers-reduced-motion)

## 🚀 Performance

- Pure CSS animations (GPU accelerated)
- Minimal DOM manipulation
- Efficient transitions
- Optimized backdrop-filter usage
- Lazy loading for heavy components

## 📦 Component Library

### Views
- **FilesView**: File management with grid layout
- **SyncView**: Sync status with stat cards
- **PeersView**: Network peer management
- **SettingsView**: Configuration interface

### Common Components
- Buttons (Primary, Secondary)
- Cards (Glass effect)
- Modals (Overlay + Card)
- Status Indicators
- Progress Bars
- Toggle Switches
- Input Fields

## 🎨 Design Tokens

All design tokens are defined in `src/styles/main.css` as CSS custom properties, making it easy to:
- Maintain consistency
- Update themes
- Create variants
- Support dark/light modes

## 🌟 Best Practices

1. **Always use CSS variables** for colors, spacing, and timing
2. **Apply glassmorphism** to elevated surfaces
3. **Add hover states** to all interactive elements
4. **Use gradient text** for important titles
5. **Include glow effects** on primary actions
6. **Animate state changes** for better UX
7. **Maintain consistent spacing** using the spacing scale
8. **Use SVG icons** instead of icon fonts or emojis
9. **Apply border radius** consistently
10. **Test on AMOLED displays** for true black verification

---

**Design System Version**: 1.0.0  
**Last Updated**: November 2025  
**Maintained by**: MSSCS Team
