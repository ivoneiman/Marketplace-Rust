
#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod marketplace_principal {
    use ink::prelude::string::String;

    use ink::prelude::vec::Vec;

    use ink::storage::Mapping;









        //#[ink(message)]
        pub enum SistemaError {
            CantidadInsuficiente,
            UsuarioNoRegistrado,
            ProductosVacios,
            NoEsRolCorrecto,
            EstadoInvalido,
            OrdenNoExiste,
        }
        use std::fmt;
        impl fmt::Display for SistemaError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    SistemaError::CantidadInsuficiente => write!(f, "Cantidad insuficiente"),
                    SistemaError::UsuarioNoRegistrado => write!(f, "Usuario no registrado"),
                    SistemaError::NoEsRolCorrecto => write!(f, "El usuario no es el rol correcto"),
                    SistemaError::ProductosVacios => write!(f, "No se han seleccionado productos"),
                    SistemaError::EstadoInvalido => write!(f, "El estado de la orden es inválido"),
                    SistemaError::OrdenNoExiste => write!(f, "La orden no existe"),
                }
            }
        }

    

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
    impl Orden {
        pub fn new(
            id: u32,
            comprador: AccountId,
            vendedor: AccountId,
            producto_id: u32,
            cantidad: u32,
        ) -> Self {
            Self {
                id,
                comprador,
                vendedor,
                producto_id,
                cantidad,
                estado: EstadoOrden::Pendiente,
                comprador_califico: false,
                vendedor_califico: false,
            }
        }

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


        






        // Despues cuando nos pongamos de acuerdo si usar el error propio arreglo esto
        #[ink(message)]
        pub fn registrar_usuario(&mut self, rol: RolUsuario) -> Result<(),String>{
            self._registrar_usuario(rol);

        }
        fn _registrar_usuario(&mut self, rol: RolUsuario) -> Result<(),String>{
            let usuario_llamador = self.env().caller(); // Devuelve AccountID
            
            // Verifico si ya existe el usuario
            if self.usuarios.contains_key(&usuario_llamador) {
                return Err("El usuario se encuentra registrado".to_string());
            }

            // Si no, creamos un nuevo usuario
            let nuevo_usuario = Usuario{
                direccion: usuario_llamador,
                rol,
                reputacion_como_comprador: 0,
                reputacion_como_vendedor: 0,
            };

            self.usuarios.insert(usuario_llamador, &nuevo_usuario);

            Ok(())
        }





        #[ink(message)]
        pub fn publicar_producto(&mut self,nombre: String,descripcion: String,precio: Balance,cantidad: u32,categoria: String,) -> Result<(), SistemaError> {
            self._publicar_producto(
                nombre,
                descripcion,
                precio,
                cantidad,
                categoria,
            )
        }
        fn _publicar_producto(&mut self,nombre: String,descripcion: String,precio: Balance,cantidad: u32,categoria: String,) -> Result<(), SistemaError> {
            let vendedor = self.env().caller();
            //Primero verifico si el usuario esta registrado en el sistema
            self.esta_registrado(vendedor)?;
            //Despues verifico si el usuario tiene el rol de vendedor
            self.es_rol_correcto(vendedor, RolUsuario::Vendedor)?;
            // Verifico si la cantidad es mayor a 0
            self.cant_suficiente(cantidad)?;
            // creo el producto
            self.crear_producto(nombre,descripcion,precio,cantidad,categoria,vendedor,)?;
            Ok(())
        }
        
        // PREGUNTAR: si éste metodo está bien o deberíamos cambiar las estructuras para que cada usuario tenga un vec con los productos propios
        // capz es más eficiente eso antes que filtar en el vec de toodos los productos.
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






        // new orden de compra

        #[ink(message)]
        pub fn crear_orden(&mut self, producto_id: u32, cantidad: u32) -> Result<u32, SistemaError> {
            let comprador = self.env().caller();

            // Verificamos si el usuario está registrado
            self.esta_registrado(comprador)?;
            // Y si es el usuario es un comprador
            self.es_rol_correcto(comprador, RolUsuario::Comprador)?;

            // Verificamos si hay productos disponibles
            let index = self.productos.iter().position(|p| p.id == producto_id)
                .ok_or(SistemaError::ProductosVacios)?;

            if self.productos[index].cantidad < cantidad {
                return Err(SistemaError::CantidadInsuficiente);
            }

            // Si todo está bien, creamos la orden
            let vendedor = self.productos[index].vendedor;
            self.productos[index].cantidad -= cantidad;

            let orden_id = self.ordenes.len() as u32;
            let orden = Orden {
                id: orden_id,
                comprador,
                vendedor,
                producto_id,
                cantidad,
                estado: EstadoOrden::Pendiente,
                comprador_aprueba_cancelacion: false,
                vendedor_aprueba_cancelacion: false,
            };
            self.ordenes.push(orden);

            Ok(orden_id)
        }



        #[ink(message)]
        pub fn marcar_orden_como_enviada(&mut self, orden_id: u32) -> Result<(), SistemaError> {
            let caller = self.env().caller();
        
            // Verificamos si el usuario está registrado
            self.esta_registrado(caller)?; 

            // Verificamos si la orden y verificamos su estado
            let orden = self.ordenes.get_mut(orden_id as usize)
                .ok_or(SistemaError::OrdenNoExiste)?;

            
            if orden.vendedor != caller {
                return Err(SistemaError::NoEsRolCorrecto);
            }
            if orden.estado != EstadoOrden::Pendiente {
                return Err(SistemaError::EstadoInvalido);
            }

            orden.estado = EstadoOrden::Enviada;
            Ok(())
        }

        #[ink(message)]
        pub fn marcar_como_recibida(&mut self, orden_id: u32) -> Result<(), SistemaError> {
            let caller = self.env().caller();

            // Verificamos si el usuario está registrado
            self.esta_registrado(caller)?;

            // Verificamos si la orden existe y su estado
            let orden = self.ordenes.get_mut(orden_id as usize)
                .ok_or(SistemaError::OrdenNoExiste)?;

            if orden.comprador != caller {
                return Err(SistemaError::NoEsRolCorrecto);
            }
            if orden.estado != EstadoOrden::Enviada {
                return Err(SistemaError::EstadoInvalido);
            }

            orden.estado = EstadoOrden::Recibida;
            Ok(())
        }


                
                    








        






        #[ink(message)]
        pub fn comprar_producto(&mut self, producto_id: u32, cantidad: u32) {
            // FALTA IMPLEMENTAR la lógica de compra
        }













// funciones auxiliares privadas:

        fn esta_registrado(&self, usuario: AccountId) -> Result<(), SistemaError> {
            if self.usuarios.contains_key(&usuario) {
                Ok(())
            } else {
                Err(SistemaError::UsuarioNoRegistrado)
            }
        }


        // Siempre antes de invocar, preguntar si existe el usuario
        fn es_rol_correcto(
            &self,
            usuario: AccountId,
            rol: RolUsuario,
        ) -> Result<(), SistemaError> {
            let user = self.usuarios.get(&usuario);
            if user.rol == rol || user.rol == RolUsuario::Ambos {
                Ok(())
            } else {
                Err(SistemaError::NoEsVendedor)
            }
        }
        fn cant_suficiente(&self, cantidad: u32) -> Result<(), SistemaError> {
            if cantidad > 0 {
                Ok(())
            } else {
                Err(SistemaError::CantidadInsuficiente)
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
        ) -> Result<(), SistemaError> {
            let id = self.productos.len() as u32; // Genera un ID único para el producto
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

        //fn tiene_rol_correcto(
        
        
    }



    
    // LUEGO DE CADA MERGE EN DEV UBISCAR LOS TEST EN EL MOD CON LOS DEMAS
    #[cfg(test)]
    mod test {

        use super::*; // Importamos todo lo que esta definido en el contrato

        // Test para comprobar el registro correcto de un usuario nuevo
        
        #[ink::test]
        fn registrar_usuario_test_funcional() {

            //Creamos una isntancia nueva del de contrato
            let mut contrato = MarketplacePrincipal::new();

            //Llamamos a la funcion registrar usuario con un rol
            let resultado = contrato.registrar_usuario(RolUsuario::Vendedor);

            //Verificamos que devuelva OK
            assert_eq!(resultado, Ok(()));

            //Obtenemos el usuario usando la dir del que llama
            let caller = contrato.env().caller(); //quien llama al contrato
            let usuario_registrado = contrato.usuarios.get(&caller);

            //Confirmamos si se guardó el usuario
            assert_eq!(usuario_registrado.is_some());

            //Verificamos los datos
            let usuario = usuario_registrado.unwrap();
            assert_eq!(usuario.rol, RolUsuario::Vendedor);
            assert_eq!(usuario.reputacion_como_comprador, 0);
            assert_eq!(usuario.reputacion_como_vendedor, 0);


        }

        // Test para comprobar que el usuario no puede registrase 2 veces
        #[ink::test]
        fn registrar_usuario_dos_veces() {
            let mut contrato = MarketplacePrincipal::new();

            //Primer registro
            let _ = contrato.registrar_usuario(RolUsuario::Comprador);

            //Segundo registro debería fallar porque ya esta registrado
            let resultado = contrato.registrar_usuario(RolUsuario::Vendedor);
            assert_eq!(resultado, Err("El usuario se encuentra regiistrado".to_string()));
               

        }

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

        }

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
            )  ;

            // Debe fallar con error de usuario no registrado (Usamos el UsuarioNoRegistrado)

            assert!(matches!(resultado, Err(SistemaError::UsuarioNoRegistrado)));


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

            assert!(matches!(resultado, Err(SistemaError::NoEsVendedor)));


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

        assert!(matches!(resultado, Err(SistemaError::CantidadInsuficiente)));


        }



    }



}
