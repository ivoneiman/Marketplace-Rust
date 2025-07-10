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
        pub fn registrar_usuario(&mut self, rol: RolUsuario) -> Result<(),String>{
            let usuario_llamador = self.env().caller(); // Devuelve AccountID
            // Verifico si ya existe el usuario
            if self.usuarios.contains(usuario_llamador){
                return Err("El usuario ya esta registrado".to_string());
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
        pub fn publicar_producto(
            &mut self,
            nombre: String,
            descripcion: String,
            precio: Balance,
            cantidad: u32,
            categoria: String,
        ) {
            // FALTA IMPLEMENTAR lógica de publicación
        }

        #[ink(message)]
        pub fn comprar_producto(&mut self, producto_id: u32, cantidad: u32) {
            // FALTA IMPLEMENTAR la lógica de compra
        }
    }

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
            assert_eq!(resultado, Ok(()))

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

        fn registrar_usuario_dos_veces() {
            let mut contrato = MarketplacePrincipal::new();

            //Primer registro
            let _ = contrato.registrar_usuario(RolUsuario::Comprador);

            //Segundo registro debería fallar porque ya esta registrado
            let resultado = contrato.registrar_usuario(RolUsuario::Vendedor);
            assert_eq!(resultado, Err("El usuario se encuentra regiistrado".to_string()));
               

        }
    }



}
