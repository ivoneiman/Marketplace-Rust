/*El presente trabajo práctico final tiene como objetivo integrar los conocimientos adquiridos durante el cursado de la materia 
Seminario de Lenguajes – Opción Rust, aplicando conceptos de programación en Rust orientados al desarrollo de contratos inteligentes 
sobre la plataforma Substrate utilizando el framework Ink!.

La consigna propone desarrollar una plataforma descentralizada de compra-venta de productos, inspirada en modelos como MercadoLibre, 
pero ejecutada completamente en un entorno blockchain. El sistema deberá dividirse en dos contratos inteligentes: uno encargado de 
gestionar la lógica principal del marketplace y otro destinado a la generación de reportes a partir de los datos públicos del primero.

El proyecto busca que el estudiante no solo practique la sintaxis y semántica de Rust, sino que también comprenda el diseño modular de 
contratos inteligentes, la separación de responsabilidades, la validación de roles y permisos, y la importancia de la transparencia, 
trazabilidad y reputación en contextos descentralizados.

Se espera que las entregas incluyan una implementación funcional, correctamente testeada, documentada y con una cobertura de pruebas mínima del 85%.

Funcionalidades
👤 Registro y gestión de usuarios
Registro de usuario con rol: Comprador, Vendedor o ambos.
Posibilidad de modificar roles posteriores.
📦 Publicación de productos
Publicar producto con nombre, descripción, precio, cantidad y categoría.
Solo disponible para usuarios con rol Vendedor.
Visualización de productos propios.
🛒 Compra y órdenes
Crear orden de compra (solo Compradores).
Al comprar: se crea la orden y se descuenta stock.
Estados de orden: pendiente, enviado, recibido, cancelada.
Solo el Vendedor puede marcar como enviado.
Solo el Comprador puede marcar como recibido o cancelada si aún está pendiente.
Cancelación requiere consentimiento mutuo.*/
#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod marketplace_principal {
    use ink::prelude::string::String;

    use ink::prelude::vec::Vec;

    use ink::storage::Mapping;

    /// Rol de usuarios
    #[derive(Debug, scale::Encode, scale::Decode, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum RolUsuario {
        Comprador,
        Vendedor,
        Ambos,
    }

    /// Posibles estados de de una orden
    #[derive(Debug, scale::Encode, scale::Decode, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum EstadoOrden {
        Pendiente,
        Enviada,
        Recibida,
        Cancelada,
    }

    /// Struct del usuario
    #[derive(Debug, scale::Encode, scale::Decode, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Usuario {
        pub direccion: AccountId,
        pub rol: RolUsuario,
        pub reputacion_como_comprador: u32,
        pub reputacion_como_vendedor: u32,
    }

    /// Struct del producto
    #[derive(Debug, scale::Encode, scale::Decode, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Producto {
        pub id: u32,
        pub nombre: String,
        pub descripcion: String,
        pub precio: Balance,
        pub cantidad: u32,
        pub categoria: String,
        pub vendedor: AccountId,
    }
    impl Producto {
        pub fn new(
            id: u32,
            nombre: String,
            descripcion: String,
            precio: Balance,
            cantidad: u32,
            categoria: String,
            vendedor: AccountId,
        ) -> Self {
            Self {
                id,
                nombre,
                descripcion,
                precio,
                cantidad,
                categoria,
                vendedor,
            }
        }
    }

    /// Struct de una orden
    #[derive(Debug, scale::Encode, scale::Decode, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Orden {
        pub id: u32,
        pub comprador: AccountId,
        pub vendedor: AccountId,
        pub producto_id: u32,
        pub cantidad: u32,
        pub estado: EstadoOrden,
        pub comprador_califico: bool,
        pub vendedor_califico: bool,
    }

    #[ink(storage)]
    // Struct de la plataforma principal
    pub struct MarketplacePrincipal {
        usuarios: Mapping<AccountId, Usuario>,
        productos: Vec<Producto>,
        ordenes: Vec<Orden>,
    }

    impl MarketplacePrincipal {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                usuarios: Mapping::default(),
                productos: Vec::new(),
                ordenes: Vec::new(),
            }
        }

        #[ink(message)]
        pub fn registrar_usuario(&mut self, rol: RolUsuario) {
            // FALTA IMPLEMENTAR 
        }
        #[ink(message)]
        // Errores personalizados para la publicación de productos
        pub enum ProductoError {
            CantidadInsuficiente,
            UsuarioNoRegistrado,
            NoEsVendedor,
        }
        use std::fmt;
        impl fmt::Display for ProductoError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    ProductoError::CantidadInsuficiente => write!(f, "Cantidad insuficiente"),
                    ProductoError::UsuarioNoRegistrado => write!(f, "Usuario no registrado"),
                    ProductoError::NoEsVendedor => write!(f, "El usuario no es un vendedor"),
                }
            }
        }

        #[ink(message)]
        pub fn publicar_producto(
            &mut self,
            nombre: String,
            descripcion: String,
            precio: Balance,
            cantidad: u32,
            categoria: String,
        ) -> Result<(), ProductoError> {
            _publicar_producto(
                self,
                nombre,
                descripcion,
                precio,
                cantidad,
                categoria,
            )
        }
            // FALTA IMPLEMENTAR lógica de publicación
        fn _publicar_producto(&mut self,
            nombre: String,
            descripcion: String,
            precio: Balance,
            cantidad: u32,
            categoria: String,
        ) -> Result<(), ProductoError> {
            let vendedor = self.env().caller();
            //Primero verifico si el usuario esta registrado en el sistema
            esta_registrado(self, vendedor)?;
            //Despues verifico si el usuario tiene el rol de vendedor
            es_vendedor(self, vendedor, RolUsuario::Vendedor)?;
            // Verifico si la cantidad es mayor a 0
            cant_suficiente(self, cantidad)?;
            // creo el producto
            crear_producto(
                self,
                nombre,
                descripcion,
                precio,
                cantidad,
                categoria,
                vendedor,
            )?;
            Ok(())
        }
        fn esta_registrado(&self, usuario: AccountId) -> Result<(), ProductoError> {
            if self.usuarios.contains_key(&usuario) {
                Ok(())
            } else {
                Err(ProductoError::UsuarioNoRegistrado)
            }
        }
        fn es_vendedor(
            &self,
            usuario: AccountId,
            rol: RolUsuario,
        ) -> Result<(), ProductoError> {
            let user = self.usuarios.get(&usuario);
            if user.rol == rol || user.rol == RolUsuario::Ambos {
                Ok(())
            } else {
                Err(ProductoError::NoEsVendedor)
            }
        }
        fn cant_suficiente(&self, cantidad: u32) -> Result<(), ProductoError> {
            if cantidad > 0 {
                Ok(())
            } else {
                Err(ProductoError::CantidadInsuficiente)
            }
        }
        fn crear_producto(
            &mut self,
            nombre: String,
            descripcion: String,
            precio: Balance,
            cantidad: u32,
            categoria: String,
            vendedor: AccountId,
        ) -> Result<(), ProductoError> {
            let id = self.productos.len() as u32 + 1; // Genera un ID único para el producto
            let nuevo_producto = Producto::new(
                id,
                nombre,
                descripcion,
                precio,
                cantidad,
                categoria,
                vendedor,
            );
            // Agrega el nuevo producto al vector de productos
            self.productos.push(nuevo_producto);
            Ok(())
        }
        #[ink(message)]
        pub fn ver_productos_propios(&self) -> Vec<Producto> {
            _ver_productos_propios(self)
        }
        fn _ver_productos_propios(&self) -> Vec<Producto> {
            let vendedor = self.env().caller();
            // Filtra los productos del vendedor actual
            self.productos
                .iter()
                .filter(|producto| producto.vendedor == vendedor)
                .cloned()
                .collect()
        }


        #[ink(message)]
        pub fn comprar_producto(&mut self, producto_id: u32, cantidad: u32) {
            // FALTA IMPLEMENTAR la lógica de compra
        }
    }
}
