use std::path::Path;
use std::env;

pub fn is_safe_path(target: &str) -> bool {
    // Usamos el directorio actual de ejecución como Sandbox por defecto
    // o una variable de entorno si el usuario la define
    let current_dir = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let allowed_base_env = env::var("SML_SANDBOX_DIR").unwrap_or_else(|_| current_dir.to_string_lossy().to_string());
    
    let base_path = Path::new(&allowed_base_env);

    let target_path = Path::new(target);
    
    // Si la ruta es absoluta, verificar que empiece con la base permitida
    if target_path.is_absolute() {
        return target_path.starts_with(base_path);
    }

    // Si es relativa, resolver basándose en el directorio actual
    if let Ok(current_dir) = env::current_dir() {
        let joined = current_dir.join(target_path);
        if let Ok(canonical) = joined.canonicalize() {
            return canonical.starts_with(base_path);
        }
        
        // Si el archivo no existe (para crearlo), verificamos el directorio padre
        if let Some(parent) = joined.parent() {
            if let Ok(canonical_parent) = parent.canonicalize() {
                 return canonical_parent.starts_with(base_path);
            }
        }
    }
    
    false
}
