#![cfg_attr(not(feature = "std"), no_std, no_main)]

    #[ink::contract]
mod marketplace_principal {

    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use ink::storage::traits::StorageLayout; // Agrega este import
    use parity_scale_codec::{Encode, Decode};
    use scale_info::TypeInfo;


        #[ink(storage)]
        pub struct MarketplacePrincipal {
            usuarios: Mapping<AccountId, Usuario>,
            productos: Vec<Producto>,
            ordenes: Vec<Orden>,
        }

        impl MarketplacePrincipal {
            // Crea una nueva instancia vacía del marketplace.
            #[ink(constructor)]
            pub fn new() -> Self {
                Self {
                    usuarios: Mapping::default(),
                    productos: Vec::new(),
                    ordenes: Vec::new(),
                }
            }

            #[ink(message)]
            pub fn registrar_usuario(&mut self, rol: RolUsuario) -> Result<(), SistemaError> {
                self.registrar_usuario_interno(rol)
            }

            fn registrar_usuario_interno(&mut self, rol: RolUsuario) -> Result<(), SistemaError> {
                let usuario_llamador = self.env().caller();
                // Verifica si el usuario es existente
                if self.usuarios.contains(&usuario_llamador) { // Cambia contains_key por contains
                    return Err(SistemaError::UsuarioExistente);
                }
                // Si no existe, crea un nuevo usuario
                let nuevo_usuario = Usuario {
                    direccion: usuario_llamador,
                    rol,
                    reputacion_como_comprador: 0,
                    reputacion_como_vendedor: 0,
                };
                self.usuarios.insert(usuario_llamador, &nuevo_usuario);
                Ok(())
            }
        }

    




    //         #[ink(message)]
    //         pub fn publicar_producto(
    //             &mut self,
    //             nombre: String,
    //             descripcion: String,
    //             precio: Balance,
    //             cantidad: u32,
    //             categoria: String,
    //         ) -> Result<(), SistemaError> {
    //             self.crear_producto_seguro(nombre, descripcion, precio, cantidad, categoria)
    //         }

            

    //         fn crear_producto_seguro(
    //             &mut self,
    //             nombre: String,
    //             descripcion: String,
    //             precio: Balance,
    //             cantidad: u32,
    //             categoria: String,
    //         ) -> Result<(), SistemaError> {
    //             let vendedor = self.env().caller();
    //             // Verifica que el vendedor esté registrado y tenga el rol adecuado
    //             self.verificar_registro(vendedor)?;
    //             self.verificar_rol(vendedor, RolUsuario::Vendedor)?;
    //             // Verifica que la cantidad sea válida
    //             self.verificar_cantidad(cantidad)?;
    //             // Agrega el producto al marketplace
    //             self.agregar_producto(nombre, descripcion, precio, cantidad, categoria, vendedor)
    //         }





    //         
    //         #[ink(message)]
    //         pub fn crear_orden(&mut self, producto_id: u32, cantidad: u32) -> Result<u32, SistemaError> {
    //             self.crear_nueva_orden(producto_id, cantidad)
    //         }
            
    //         /// Crea una nueva orden de compra para un producto existente.
    //         fn crear_nueva_orden(&mut self, producto_id: u32, cantidad: u32) -> Result<u32, SistemaError> {
    //             let comprador = self.env().caller();
    //             self.verificar_registro(comprador)?;
    //             self.verificar_rol(comprador, RolUsuario::Comprador)?;

    //             let producto = self.obtener_producto_mut(producto_id)?;
    //             if producto.cantidad < cantidad {
    //                 return Err(SistemaError::CantidadInsuficiente);
    //             }

    //             producto.cantidad -= cantidad;
    //             self.crear_y_emitir_orden(comprador, producto.vendedor, producto_id, cantidad)
    //         }



    
    //         #[ink(message)]
    //         pub fn marcar_orden_como_enviada(&mut self, orden_id: u32) -> Result<(), SistemaError> {
    //             self.actualizar_estado_orden(orden_id, EstadoOrden::Enviada)
    //         }

    
    //         #[ink(message)]
    //         pub fn marcar_como_recibida(&mut self, orden_id: u32) -> Result<(), SistemaError> {
    //             self.actualizar_estado_orden(orden_id, EstadoOrden::Recibida)
    //         }

            


    //         fn actualizar_estado_orden(&mut self, orden_id: u32, nuevo_estado: EstadoOrden) -> Result<(), SistemaError> {
    //             let caller = self.env().caller();
    //             // Tiene que estar registrado
    //             self.verificar_registro(caller)?;

    //             let orden = self.obtener_orden_mut(orden_id)?;
    //             self.verificar_permiso_orden(caller, orden, nuevo_estado)?;

    //             let _estado_anterior = orden.estado;
    //             orden.estado = nuevo_estado;

    //             // self.emitir_evento_estado(orden_id, orden.comprador, nuevo_estado);
    //             Ok(())
    //         }


    //         // --- Funciones auxiliares ---


    //         fn verificar_registro(&self, usuario: AccountId) -> Result<(), SistemaError> {
    //             if !self.usuarios.contains(&usuario) { // Cambia contains_key por contains
    //                 Err(SistemaError::UsuarioNoRegistrado)
    //             } else {
    //                 Ok(())
    //             }
    //         }

    //         fn verificar_usuario_existente(&self, usuario: AccountId) -> Result<(), SistemaError> {
    //             if self.usuarios.contains(&usuario) { // Cambia contains_key por contains
    //                 Err(SistemaError::UsuarioExistente)
    //             } else {
    //                 Ok(())
    //             }
    //         }

    //         fn verificar_rol(&self, usuario: AccountId, rol_requerido: RolUsuario) -> Result<(), SistemaError> {
    //             let usuario_data = self.usuarios.get(&usuario)
    //                 .ok_or(SistemaError::UsuarioNoRegistrado)?;

    //             match (usuario_data.rol, rol_requerido) {
    //                 (RolUsuario::Ambos, _) => Ok(()),
    //                 (rol, requerido) if rol == requerido => Ok(()),
    //                 _ => Err(SistemaError::NoEsRolCorrecto),
    //             }
    //         }

    //         fn verificar_cantidad(&self, cantidad: u32) -> Result<(), SistemaError> {
    //             if cantidad <= 0 {
    //                 Err(SistemaError::CantidadInsuficiente)
    //             } else {
    //                 Ok(())
    //             }
    //         }

    //         fn agregar_producto(
    //             &mut self,
    //             nombre: String,
    //             descripcion: String,
    //             precio: Balance,
    //             cantidad: u32,
    //             categoria: String,
    //             vendedor: AccountId,
    //         ) -> Result<(), SistemaError> {
    //             let id = self.productos.len() as u32;
    //             let nuevo_producto = Producto::new(id, nombre, descripcion, precio, cantidad, categoria, vendedor);
    //             self.productos.push(nuevo_producto);
    //             Ok(())
    //         }

    //         fn obtener_producto_mut(&mut self, id: u32) -> Result<&mut Producto, SistemaError> {
    //             self.productos
    //                 .iter_mut()
    //                 .find(|p| p.id == id)
    //                 .ok_or(SistemaError::ProductosVacios)
    //         }

    //         fn crear_y_emitir_orden(
    //             &mut self,
    //             comprador: AccountId,
    //             vendedor: AccountId,
    //             producto_id: u32,
    //             cantidad: u32
    //         ) -> Result<u32, SistemaError> {
    //             let id = self.ordenes.len() as u32;
    //             let nueva_orden = Orden::new(id, comprador, vendedor, producto_id, cantidad);
    //             self.ordenes.push(nueva_orden.clone());
    //             // self.emitir_evento_creacion(nueva_orden);
    //             Ok(id)
    //         }

    //         fn obtener_orden_mut(&mut self, id: u32) -> Result<&mut Orden, SistemaError> {
    //             self.ordenes
    //                 .get_mut(id as usize)
    //                 .ok_or(SistemaError::OrdenNoExiste)
    //         }

    //         fn verificar_permiso_orden(
    //             &self,
    //             caller: AccountId,
    //             orden: &Orden,
    //             nuevo_estado: EstadoOrden
    //         ) -> Result<(), SistemaError> {
    //             match nuevo_estado {
    //                 EstadoOrden::Enviada if caller != orden.vendedor => Err(SistemaError::NoEsRolCorrecto),
    //                 EstadoOrden::Recibida if caller != orden.comprador => Err(SistemaError::NoEsRolCorrecto),
    //                 _ => self.verificar_transicion_estado(orden.estado, nuevo_estado),
    //             }
    //         }

    //         fn verificar_transicion_estado(
    //             &self,
    //             actual: EstadoOrden,
    //             nuevo: EstadoOrden
    //         ) -> Result<(), SistemaError> {
    //             match (actual, nuevo) {
    //                 (EstadoOrden::Pendiente, EstadoOrden::Enviada) => Ok(()),
    //                 (EstadoOrden::Enviada, EstadoOrden::Recibida) => Ok(()),
    //                 _ => Err(SistemaError::EstadoInvalido),
    //             }
    //         }
    //     }

















    // ────────────────
    // ENUMS
    // ────────────────

    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo, StorageLayout)]
    pub enum RolUsuario {
        Comprador,
        Vendedor,
        Ambos,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo)]
    pub enum EstadoOrden {
        Pendiente,
        Enviada,
        Recibida,
        Cancelada,
    }

    
    // ────────────────
    // ERRORES DEL SISTEMA
    // ────────────────


    #[derive(Debug, Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
    pub enum SistemaError {
        CantidadInsuficiente,
        UsuarioNoRegistrado,
        ProductosVacios,
        NoEsRolCorrecto,
        EstadoInvalido,
        OrdenNoExiste,
        UsuarioExistente
    }
    impl core::fmt::Display for SistemaError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                SistemaError::CantidadInsuficiente => write!(f, "Cantidad insuficiente"),
                SistemaError::UsuarioNoRegistrado => write!(f, "El usuario no se encuentra registrado"),
                SistemaError::NoEsRolCorrecto => write!(f, "El usuario no es del rol correcto"),
                SistemaError::ProductosVacios => write!(f, "No se encontraron productos"),
                SistemaError::EstadoInvalido => write!(f, "El estado de la orden es inválido"),
                SistemaError::OrdenNoExiste => write!(f, "La orden no existe"),
                SistemaError::UsuarioExistente => write!(f, "El usuario ya está registrado"),

            }
        }
    }



    // ────────────────
    // ESTRUCTURAS PRINCIPALES
    // ────────────────


    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo, StorageLayout)]
    pub struct Usuario {
        pub direccion: AccountId,
        pub rol: RolUsuario,
        pub reputacion_como_comprador: u32,
        pub reputacion_como_vendedor: u32,
    }

 
    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo)]
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
        pub fn new(id: u32, nombre: String, descripcion: String, precio: Balance, cantidad: u32, categoria: String, vendedor: AccountId) -> Self {
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

    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo)]
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
        pub fn new(id: u32, comprador: AccountId, vendedor: AccountId, producto_id: u32, cantidad: u32) -> Self {
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



















    // LUEGO DE CADA MERGE EN DEV, UBICAR LOS TESTS EN EL MÓDULO CON LOS DEMÁS
    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test;

        #[ink::test]
        fn registrar_usuario_test_funcional() {
            let mut contrato = MarketplacePrincipal::new();

            // Simulamos que el caller es "Alice"
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

            // Llamamos a la función registrar_usuario con un rol
            let resultado = contrato.registrar_usuario(RolUsuario::Vendedor);

            // Verificamos que devuelva OK
            assert_eq!(resultado, Ok(()));

            // Obtenemos el usuario usando la dirección del caller
            let usuario_registrado = contrato.usuarios.get(&accounts.alice);

            // Confirmamos si se guardó el usuario
            assert!(usuario_registrado.is_some());

            // Verificamos los datos
            let usuario = usuario_registrado.unwrap();
            assert_eq!(usuario.rol, RolUsuario::Vendedor);
            assert_eq!(usuario.reputacion_como_comprador, 0);
            assert_eq!(usuario.reputacion_como_vendedor, 0);
        }
    }
        // #[ink::test]
        // fn registrar_usuario_dos_veces() {
        //     let mut contrato = MarketplacePrincipal::new();

        //     let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
        //     test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        //     // Primer registro
        //     let _ = contrato.registrar_usuario(RolUsuario::Comprador);

        //     // Segundo registro debería fallar porque ya está registrado
        //     let resultado = contrato.registrar_usuario(RolUsuario::Vendedor);
        //     assert_eq!(resultado, Err(SistemaError::UsuarioYaRegistrado));
        // }

        // fn setup_contract_con_vendedor() -> MarketplacePrincipal {
        //     let mut contrato = MarketplacePrincipal::new();
        //     let caller = AccountId::from([0x01; 32]);

        //     // Simulamos que "caller" es quien está invocando el contrato
        //     test::set_caller::<ink::env::DefaultEnvironment>(caller);

        //     let usuario = Usuario {
        //         direccion: caller,
        //         rol: RolUsuario::Vendedor,
        //         reputacion_como_comprador: 0,
        //         reputacion_como_vendedor: 0,
        //     };

        //     contrato.usuarios.insert(caller, &usuario);

        //     contrato
        // }

        // #[ink::test]
        // fn test_publicar_producto_ok() {
        //     let mut contrato = setup_contract_con_vendedor();

        //     let resultado = contrato.publicar_producto(
        //         "Celular".to_string(),
        //         "Un buen celular".to_string(),
        //         1000,
        //         5,
        //         "Tecnología".to_string(),
        //     );

        //     assert!(resultado.is_ok());
        //     assert_eq!(contrato.productos.len(), 1);

        //     let producto = &contrato.productos[0];
        //     assert_eq!(producto.nombre, "Celular");
        //     assert_eq!(producto.precio, 1000);
        // }

        // #[ink::test]
        // fn test_usuario_no_registrado() {
        //     let mut contrato = MarketplacePrincipal::new();

        //     let caller = AccountId::from([0x02; 32]);
        //     test::set_caller::<ink::env::DefaultEnvironment>(caller);

        //     let resultado = contrato.publicar_producto(
        //         "Producto".to_string(),
        //         "Sin registro".to_string(),
        //         500,
        //         1,
        //         "Otros".to_string(),
        //     );

        //     assert!(matches!(resultado, Err(SistemaError::UsuarioNoRegistrado)));
        // }

        // #[ink::test]
        // fn test_usuario_no_es_vendedor() {
        //     let mut contrato = MarketplacePrincipal::new();

        //     let caller = AccountId::from([0x03; 32]);
        //     test::set_caller::<ink::env::DefaultEnvironment>(caller);

        //     let usuario = Usuario {
        //         direccion: caller,
        //         rol: RolUsuario::Comprador, // Rol no válido para publicar productos
        //         reputacion_como_comprador: 0,
        //         reputacion_como_vendedor: 0,
        //     };
        //     contrato.usuarios.insert(caller, &usuario);

        //     let resultado = contrato.publicar_producto(
        //         "Producto".to_string(),
        //         "No autorizado".to_string(),
        //         100,
        //         2,
        //         "Otros".to_string(),
        //     );

        //     assert!(matches!(resultado, Err(SistemaError::NoEsRolCorrecto)));
        // }

        // #[ink::test]
        // fn test_cantidad_insuficiente() {
        //     let mut contrato = setup_contract_con_vendedor();

        //     let resultado = contrato.publicar_producto(
        //         "Producto".to_string(),
        //         "Cantidad cero".to_string(),
        //         100,
        //         0, // Cantidad inválida
        //         "Otros".to_string(),
        //     );

        //     assert!(matches!(resultado, Err(SistemaError::CantidadInsuficiente)));
        // }
}