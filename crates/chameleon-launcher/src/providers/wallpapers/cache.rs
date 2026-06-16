use blake3::Hasher;
use image::{DynamicImage, ImageReader};
use std::env::home_dir;
use std::path::{Path, PathBuf};
use std::{fs, io};

/// Генерирует безопасное имя файла для кэша.
/// Формат: <32-символа хэша пути>-<оригинальное имя>-<ширина>x<высота>.png
pub fn cache_filename(path: &Path, width: u32, height: u32) -> String {
    let path_str = path.to_string_lossy();
    let hash = {
        let mut hasher = Hasher::new();
        hasher.update(path_str.as_bytes());
        hasher.finalize().to_hex().to_string()
    };
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default()
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != '.', "_"); // чистим от проблемных символов
    format!("{}-{}-{}x{}.png", hash, name, width, height)
}

/// Получает директорию кэша приложения: ~/.cache/chameleon/thumbnails
pub fn cache_dir() -> PathBuf {
    home_dir().unwrap().join(".cache/chameleon/thumbnails")
}

/// Создаёт директорию кэша, если её нет.
pub fn ensure_cache_dir() -> io::Result<()> {
    let dir = cache_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(())
}

/// Загружает превью из кэша (если существует) в виде `DynamicImage`.
pub fn load_from_cache(
    path: &Path,
    width: u32,
    height: u32,
) -> Option<DynamicImage> {
    let filename = cache_filename(path, width, height);
    let cache_path = cache_dir().join(filename);
    if cache_path.exists() {
        tracing::info!("Загружаем из кэша: {:?}", cache_path);
        match ImageReader::open(&cache_path).ok()?.decode() {
            Ok(img) => return Some(img),
            Err(e) => tracing::warn!(
                "Ошибка декодирования кэша {}: {}",
                cache_path.display(),
                e
            ),
        }
    }
    None
}

/// Сохраняет превью в кэш (перезаписывает, если существует).
pub fn save_to_cache(
    preview: &DynamicImage,
    path: &Path,
    width: u32,
    height: u32,
) -> io::Result<()> {
    ensure_cache_dir()?;
    let filename = cache_filename(path, width, height);
    let cache_path = cache_dir().join(filename);

    // Сохраняем как PNG (без потерь, маленький размер для превью)
    preview
        .save(&cache_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    tracing::info!("Сохранено в кэш: {:?}", cache_path);
    Ok(())
}
