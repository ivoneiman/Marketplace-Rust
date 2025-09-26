# 🛒 Marketplace Descentralizado en Rust + Ink!

Trabajo Práctico Final para la materia Seminario de Lenguajes - Rust
## Implementación de un marketplace descentralizado tipo MercadoLibre sobre blockchain

### 🌟 Características principales
👥 Gestión de Usuarios
- Registro con roles diferenciados (🛍️ Comprador / 🏪 Vendedor)
- Perfiles verificables en blockchain
- Sistema de reputación basado en transacciones

🛍️ Sistema de Productos
 Publicación de artículos

💰 Transacciones Seguras
Sistema de órdenes con estados:
- ⏳ Pendiente
- 🚚 Enviado
- ✅ Recibido

📃 Diagrama de clases <br>
[diagrama-readme.pdf](https://github.com/user-attachments/files/22568051/diagrama-readme.pdf)

🌐 Despliegue
- Contrato desplegado en Shibuya Testnet (Polkadot)
- Interfaz web compatible con wallets como Polkadot.js

## 🛠️ Configuración Técnica
### 📋 Requisitos Previos
- Rust Nightly (2024-05-20)
- cargo-contract 4.1.3
- Substrate Contracts Node (para desarrollo local)

### ⚙️ Instalación
Configurar toolchain:
echo '[toolchain]
channel = "nightly-2024-05-20"
components = ["rust-src"]' > rust-toolchain.toml

Instalar dependencias:
rustup target add wasm32-unknown-unknown
cargo install cargo-contract --version 4.1.3

Configurar entorno:
rustup component add rust-src --toolchain nightly-2024-05-20

🏗️ Compilación
cargo contract build --release

📦 Artefactos generados en target/ink/:
marketplace.wasm (código ejecutable)
marketplace.contract (ABI + WASM)
metadata.json (interfaz del contrato)

🧪 Testing
🔬 Tests Unitarios
cargo test --lib
✅ Cobertura mínima garantizada: 85%
📊 Ver reporte: cargo tarpaulin --out Html

🌐 Tests End-to-End
cargo test --features e2e-tests

Pruebas que incluyen:
-Interacción con wallet
-Transacciones reales
-Simulación de red

🚀 Despliegue
En Testnet (Shibuya):

cargo contract upload --suri //Alice --url wss://shibuya-rpc.dwellir.com

Localmente:
substrate-contracts-node --dev
cargo contract instantiate --constructor new --args false --suri //Alice



## 📚 Documentación Adicional
📄 Documentación técnica
🖥️ Interfaz web
📊 Diagrama de arquitectura

