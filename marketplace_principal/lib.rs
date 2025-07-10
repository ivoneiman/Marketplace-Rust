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

        // Errores personalizados para la publicación de productos
        #[ink(message)]
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


    // LUEGO DEL MERGE EN DEV UBISCAR LOS TEST EN EL MODULO DE LOS DEMAS


    // Esta función que prepararr un contrato con un usuario registrado como Vendedor
    fn setup_contract_con_vendedor() -> MarketplacePrincipal {
        let mut contrato = MarketplacePrincipal::new();

        // Creamos una cuenta simulada con una dirección inventada
        let caller = AccountId::from([0x01; 32]);

        // Esta línea simula que "caller" es quien está invocando el contrato
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(caller);

        // Registramos a este usuario en el sistema con rol de Vendedorr

        let usuario = Usuario {
            direccion: caller,
            rol: RolUsuario::Vendedor,
            reputacion_como_comprador: 0,
            reputacion_como_vendedor: 0,
        };

        // Insertamos al usuario en la estructura de datos del contrato
        contrato.usuarios.insert(caller, &usuario);

        contrato



    //  Test que verifica que se puede publicar un producto correctamente
    #[ink::test]
    fn test_publicar_producto_ok() {

        // Preparamos un contrato con un vendedor válido

        let mut contrato = setup_contract_con_vendedor();

        // Llamamos a la función "publicar producto" con datos válidos
        let resultado = contrato.publicar_producto(
            "Celular".to_string(),
            "Un buen celular".to_string(),
            1000,
            5,
            "Tecnología".to_string(),
        );

        // Chequeamos que la operación fue exitosa

        assert!(resultado.is_ok());

        // Vemos si se agregó exactamente un productoo

        assert_eq!(contrato.productos.len(), 1);

        // Chequeamos los datos del producto publicado
        let producto = &contrato.productos[0];
        assert_eq!(producto.nombre, "Celular");
        assert_eq!(producto.precio, 1000);


    }



    // Test falla si el usuario no está registrado
    #[ink::test]
    fn test_usuario_no_registrado() {
        let mut contrato = MarketplacePrincipal::new();

        // Simulamos que quien llama es unm usuario no registrado

        let caller = AccountId::from([0x02; 32]);
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(caller);

        // Intentamos publicar un producto sin estar registrado

        let resultado = contrato.publicar_producto(
            "Producto".to_string(),
            "Sin registro".to_string(),
            500,
            1,
            "Otros".to_string(),
        );

        // Debe fallar con error de usuario no registrado (Usamos el UsuarioNoRegistrado)

        assert!(matches!(resultado, Err(ProductoError::UsuarioNoRegistrado)));


    }

    // Falla si el usuario está registrado pero no tiene el rol adecuado
    #[ink::test]

    fn test_usuario_no_es_vendedor() {


        let mut contrato = MarketplacePrincipal::new();

        // Simulamos "caller"
        let caller = AccountId::from([0x03; 32]);
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(caller);

        // Registramos al usuario como Comprador (no como Vendedor)
        let usuario = Usuario {
            direccion: caller,
            rol: RolUsuario::Comprador, // Ponemos un Rol no válido para publicar productos 
            reputacion_como_comprador: 0,
            reputacion_como_vendedor: 0,
        };
        contrato.usuarios.insert(caller, &usuario);

        let resultado = contrato.publicar_producto(
            "Producto".to_string(),
            "No autorizado".to_string(),
            100,
            2,
            "Otros".to_string(),
        );

        assert!(matches!(resultado, Err(ProductoError::NoEsVendedor)));


    }

    // Falla si la cantidad del producto es 0
    #[ink::test]
    fn test_cantidad_insuficiente() {


        let mut contrato = setup_contract_con_vendedor();

        let resultado = contrato.publicar_producto(
            "Producto".to_string(),
            "Cantidad cero".to_string(),
            100,
            0, // Ponemos una cantidad inválidaa
            "Otros".to_string(),
        );

        assert!(matches!(resultado, Err(ProductoError::CantidadInsuficiente)));


    }


}
