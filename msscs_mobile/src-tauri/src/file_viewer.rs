// File viewer module - Open files with system apps
use std::path::Path;

pub fn open_with_system(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        open_android(path)
    }
    
    #[cfg(not(target_os = "android"))]
    {
        open_desktop(path)
    }
}

#[cfg(target_os = "android")]
fn open_android(path: &Path) -> Result<(), String> {
    use jni::objects::{JObject, JString, JValue};
    use jni::JavaVM;
    
    // Get JNI environment
    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }
        .map_err(|e| format!("Failed to get JavaVM: {}", e))?;
    
    let env = vm.get_env()
        .map_err(|e| format!("Failed to get JNI env: {}", e))?;
    
    // Get file path as JString
    let path_str = path.to_str().ok_or("Invalid path")?;
    let j_path = env.new_string(path_str)
        .map_err(|e| format!("Failed to create JString: {}", e))?;
    
    // Get MIME type
    let mime_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();
    let j_mime = env.new_string(mime_type)
        .map_err(|e| format!("Failed to create MIME JString: {}", e))?;
    
    // Call Android Intent to open file
    let activity = unsafe { JObject::from_raw(ctx.context().cast()) };
    
    // Create Intent
    let intent_class = env.find_class("android/content/Intent")
        .map_err(|e| format!("Failed to find Intent class: {}", e))?;
    
    let action_view = env.get_static_field(
        intent_class,
        "ACTION_VIEW",
        "Ljava/lang/String;"
    ).map_err(|e| format!("Failed to get ACTION_VIEW: {}", e))?;
    
    let intent = env.new_object(
        intent_class,
        "(Ljava/lang/String;)V",
        &[action_view]
    ).map_err(|e| format!("Failed to create Intent: {}", e))?;
    
    // Set data and type
    let uri_class = env.find_class("android/net/Uri")
        .map_err(|e| format!("Failed to find Uri class: {}", e))?;
    
    let uri = env.call_static_method(
        uri_class,
        "parse",
        "(Ljava/lang/String;)Landroid/net/Uri;",
        &[JValue::Object(&JObject::from(j_path))]
    ).map_err(|e| format!("Failed to parse URI: {}", e))?;
    
    env.call_method(
        intent,
        "setDataAndType",
        "(Landroid/net/Uri;Ljava/lang/String;)Landroid/content/Intent;",
        &[uri, JValue::Object(&JObject::from(j_mime))]
    ).map_err(|e| format!("Failed to set data and type: {}", e))?;
    
    // Add FLAG_GRANT_READ_URI_PERMISSION
    let flags = 0x00000001; // FLAG_GRANT_READ_URI_PERMISSION
    env.call_method(
        intent,
        "addFlags",
        "(I)Landroid/content/Intent;",
        &[JValue::Int(flags)]
    ).map_err(|e| format!("Failed to add flags: {}", e))?;
    
    // Start activity
    env.call_method(
        activity,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(&intent)]
    ).map_err(|e| format!("Failed to start activity: {}", e))?;
    
    Ok(())
}

#[cfg(not(target_os = "android"))]
fn open_desktop(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/C", "start", "", path.to_str().unwrap()])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

pub fn get_file_icon(extension: &str) -> &'static str {
    match extension.to_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" => "🖼️",
        "mp4" | "avi" | "mkv" | "mov" | "webm" | "flv" => "🎥",
        "mp3" | "wav" | "flac" | "ogg" | "m4a" => "🎵",
        "pdf" => "📄",
        "doc" | "docx" => "📝",
        "xls" | "xlsx" => "📊",
        "ppt" | "pptx" => "📽️",
        "zip" | "rar" | "7z" | "tar" | "gz" => "📦",
        "txt" | "md" | "log" => "📃",
        "json" | "xml" | "yaml" | "toml" => "⚙️",
        "rs" | "py" | "js" | "ts" | "java" | "c" | "cpp" | "go" => "💻",
        _ => "📁",
    }
}
