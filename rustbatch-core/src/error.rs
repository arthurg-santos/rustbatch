use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("formato não suportado: {0}")]
    UnsupportedFormat(String),

    #[error("arquivo corrompido ou ilegível: {path}")]
    CorruptFile {
        path: PathBuf,
        #[source]
        source: image::ImageError,
    },

    #[error("dimensões inválidas: {width}x{height} (máximo permitido: {max} px por lado)")]
    InvalidDimensions { width: u32, height: u32, max: u32 },

    #[error("o arquivo excede o limite de memória: {0} pixels")]
    TooLarge(u64),

    #[error("conflito de nome de saída: {0}")]
    NameConflict(PathBuf),

    #[error("permissão negada: {0}")]
    PermissionDenied(PathBuf),

    #[error("caminho de saída inválido: {0}")]
    InvalidOutputPath(PathBuf),

    #[error("operação cancelada pelo usuário")]
    Cancelled,

    #[error("erro de E/S: {path}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("erro ao codificar imagem: {0}")]
    Encode(String),

    #[error("erro ao decodificar imagem: {0}")]
    Decode(String),

    #[error("marca d'água de imagem inválida: {0}")]
    InvalidWatermark(String),
}

impl CoreError {
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io { path: path.into(), source }
    }
}