# RustBatch

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![GTK4](https://img.shields.io/badge/GTK4-4A86CF?style=for-the-badge&logo=gtk&logoColor=white)
![License](https://img.shields.io/badge/license-MIT-blue.svg?style=for-the-badge)

O **RustBatch** é um software desktop nativo, open-source e multiplataforma (Windows e Linux) para processamento de imagens em lote. Desenvolvido em Rust com interface gráfica GTK4, ele foi projetado para ser rápido, seguro e não destrutivo.

## Funcionalidades

- **Importação flexível:** Arraste e solte arquivos ou selecione pastas inteiras.
- **Redimensionamento:** Redimensione por dimensões exatas, porcentagem ou ajuste para caber dentro de limites, preservando a proporção e a orientação EXIF.
- **Conversão de Formatos:** Suporte a JPEG, PNG, WebP (lossless), AVIF, TIFF, BMP e GIF.
- **Compressão Inteligente:** Controle de qualidade para JPEG e níveis de compressão para PNG.
- **Filtros e Ajustes:** Brilho, contraste, saturação, escala de cinza, sépia, desfoque, nitidez e inversão.
- **Marca D'água:** Aplique texto ou imagem com controle de opacidade, tamanho e posicionamento.
- **Renomeação em Massa:** Prefixos, sufixos, substituições de texto, numeração sequencial e inclusão de data.
- **Processamento Seguro:** Pipeline não destrutiva, processamento paralelo seguro e escrita atômica (evita arquivos corrompidos em caso de cancelamento).
- **Pré-visualização:** Visualize o resultado antes de processar.

## Arquitetura

O projeto está dividido em um *Workspace* Cargo com duas crates principais:

- `rustbatch-core`: Biblioteca independente que contém toda a lógica de negócios, modelos, pipeline de processamento de imagem e I/O. Pode ser testada sem a interface gráfica.
- `rustbatch-ui`: Aplicação executável que contém a interface GTK4, construída com foco em usabilidade, acessibilidade e integração com o sistema operacional.

## Como Compilar e Rodar

### Pré-requisitos

#### Linux (Fedora / RHEL)
```bash
sudo dnf install gtk4-devel gcc pkg-config
```

#### Linux (Ubuntu / Debian)
```bash
sudo apt install libgtk-4-dev build-essential pkg-config
```

#### Windows
Recomenda-se o uso do ambiente **MSYS2** (UCRT64) ou a instalação do GTK4 via **vcpkg/gvsbuild**.
```bash
pacman -S mingw-w64-ucrt-x86_64-gtk4 mingw-w64-ucrt-x86_64-toolchain
```

### Executando

Clone o repositório e execute:

```bash
git clone https://github.com/SEU_USUARIO/rustbatch.git
cd rustbatch
cargo run --release
```

## Licença

Este projeto está licenciado sob a Licença MIT. Consulte o arquivo [LICENSE](LICENSE) para mais detalhes.