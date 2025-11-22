# Alternative Approach: Capacitor-based Android App

## Issue with Current Approach

Tauri v1.5 doesn't support Android (desktop only). Tauri v2 has Android support but is still in beta and has configuration issues.

## Recommended Solution: Use Capacitor

Capacitor is a mature, production-ready framework for building native mobile apps with web technologies.

### Advantages:
- ✅ Stable and production-ready
- ✅ Official Android support
- ✅ Works with Vue.js
- ✅ Can call native Android APIs
- ✅ Easy to build APK
- ✅ Used by Ionic and many production apps

## Quick Setup with Capacitor

```bash
# 1. Install Capacitor
npm install @capacitor/core @capacitor/cli @capacitor/android

# 2. Initialize Capacitor
npx cap init "MSSCS Mobile" "com.msscs.mobile"

# 3. Build web app
npm run build

# 4. Add Android platform
npx cap add android

# 5. Sync web app to Android
npx cap sync

# 6. Open in Android Studio
npx cap open android

# 7. Build APK in Android Studio
# Build → Build Bundle(s) / APK(s) → Build APK(s)
```

## Alternative: Use Cordova

Cordova is even more mature and simpler:

```bash
# 1. Install Cordova
npm install -g cordova

# 2. Create Cordova project
cordova create msscs-mobile-cordova com.msscs.mobile "MSSCS Mobile"

# 3. Add Android platform
cordova platform add android

# 4. Build APK
cordova build android

# Output: platforms/android/app/build/outputs/apk/debug/app-debug.apk
```

## Simplest Solution: Progressive Web App (PWA)

Since MSSCS Mobile is primarily a web interface to the backend, you can:

1. Build as PWA
2. Users install from browser
3. Works offline
4. No app store needed

```bash
# Add PWA support
npm install vite-plugin-pwa -D

# Build
npm run build

# Deploy to web server
# Users visit URL and "Add to Home Screen"
```

## Recommendation

For fastest results, I recommend:

1. **Short term**: Build as PWA (works immediately on Android)
2. **Medium term**: Use Capacitor for native APK
3. **Long term**: Wait for Tauri v2 stable release

Would you like me to:
- A) Set up Capacitor for native APK
- B) Set up PWA for immediate use
- C) Create a hybrid approach (PWA + Capacitor)
