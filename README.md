# 🛒 Marketplace Descentralizado en Rust + Ink!

Trabajo Práctico Final para la materia Seminario de Lenguajes - Rust
## Implementación de un marketplace descentralizado tipo MercadoLibre sobre blockchain

### Características principales
👥 Gestión de Usuarios
- Registro con roles diferenciados (🛍️ Comprador / 🏪 Vendedor)
- Perfiles verificables en blockchain
- Sistema de reputación basado en transacciones

Sistema de Productos
 Publicación de artículos

Transacciones Seguras
Sistema de órdenes con estados:
- ⏳ Pendiente
- 🚚 Enviado
- ✅ Recibido
Disputas para cancelar pedidos 

📃 Diagrama de clases <br>
[diagrama-readme.pdf](https://github.com/user-attachments/files/22568051/diagrama-readme.pdf)

🌐 Despliegue
- Contrato desplegado en Shibuya Testnet (Polkadot)
- Interfaz web compatible con wallets como Polkadot.js

## 🛠️ Configuración Técnica
### 📋 Requisitos Previos
- Rust Nightly
- cargo-contract 4.1.3
- Substrate Contracts Node (para desarrollo local)


✅ Cobertura mínima garantizada > 85%
📊 Ver reporte: cargo tarpaulin --out Html
🌐 Tests End-to-End para el contrato de reportes view


