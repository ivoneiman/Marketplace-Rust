#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod marketplace_principal {
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

    // ────────────────
    // EVENTOS
    // ────────────────

    /// Evento emitido cuando se crea una nueva orden.
    #[ink(event)]
    pub struct OrdenCreada {
        /// ID de la orden.
        #[ink(topic)]
        id: u32,
        /// Cuenta del comprador.
        #[ink(topic)]
        comprador: AccountId,
        /// Cuenta del vendedor.
        vendedor: AccountId,
        /// ID del producto solicitado.
        producto_id: u32,
        /// Cantidad comprada.
        cantidad: u32,
    }

    /// Evento emitido cuando cambia el estado de una orden.
    #[ink(event)]
    pub struct EstadoOrdenCambiado {
        /// ID de la orden.
        #[ink(topic)]
        id: u32,
        /// Cuenta del comprador (para tracking).
        #[ink(topic)]
        comprador: AccountId,
        /// Nuevo estado de la orden.
        nuevo_estado: EstadoOrden,
    }

    // ────────────────
    // ERRORES DEL SISTEMA
    // ────────────────

    /// Representa posibles errores que pueden surgir en el sistema.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum SistemaError {
        CantidadInsuficiente,
        UsuarioNoRegistrado,
        ProductosVacios,
        NoEsRolCorrecto,
        EstadoInvalido,
        OrdenNoExiste,
    }

    impl core::fmt::Display for SistemaError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                SistemaError::CantidadInsuficiente => write!(f, "Cantidad insuficiente"),
                SistemaError::UsuarioNoRegistrado => write!(f, "Usuario no registrado"),
                SistemaError::NoEsRolCorrecto => write!(f, "El usuario no es del rol correcto"),
                SistemaError::ProductosVacios => write!(f, "No se encontraron productos"),
                SistemaError::EstadoInvalido => write!(f, "El estado de la orden es inválido"),
                SistemaError::OrdenNoExiste => write!(f, "La orden no existe"),
            }
        }
    }

    // ────────────────
    // ENUMS
    // ────────────────

    /// Rol que puede tener un usuario dentro del sistema.
    #[derive(Debug, scale::Encode, scale::Decode, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum RolUsuario {
        Comprador,
        Vendedor,
        Ambos,
    }

    /// Posibles estados que puede tener una orden de compra.
    #[derive(Debug, scale::Encode, scale::Decode, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum EstadoOrden {
        Pendiente,
        Enviada,
        Recibida,
        Cancelada,
    }

    // ────────────────
    // ESTRUCTURAS PRINCIPALES
    // ────────────────

    /// Representa un usuario del marketplace.
    #[derive(Debug, scale::Encode, scale::Decode, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Usuario {
        /// Dirección de la cuenta del usuario.
        pub direccion: AccountId,
        /// Rol asignado al usuario.
        pub rol: RolUsuario,
        /// Reputación acumulada como comprador.
        pub reputacion_como_comprador: u32,
        /// Reputación acumulada como vendedor.
        pub reputacion_como_vendedor: u32,
    }

    /// Representa un producto publicado en el marketplace.
    #[derive(Debug, scale::Encode, scale::Decode, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Producto {
        /// ID único del producto.
        pub id: u32,
        pub nombre: String,
        pub descripcion: String,
        pub precio: Balance,
        pub cantidad: u32,
        pub categoria: String,
        /// Cuenta del vendedor que publicó el producto.
        pub vendedor: AccountId,
    }

    impl Producto {
        /// Crea una nueva instancia de producto.
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

    /// Representa una orden de compra realizada por un usuario.
    #[derive(Debug, scale::Encode, scale::Decode, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Orden {
        /// ID único de la orden.
        pub id: u32,
        pub comprador: AccountId,
        pub vendedor: AccountId,
        pub producto_id: u32,
        pub cantidad: u32,
        pub estado: EstadoOrden,
        /// Indica si el comprador ya calificó.
        pub comprador_califico: bool,
        /// Indica si el vendedor ya calificó.
        pub vendedor_califico: bool,
    }

    impl Orden {
        /// Crea una nueva orden con estado pendiente y sin calificaciones.
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
}












    /// Contrato principal del marketplace.
/// Permite registrar usuarios, publicar productos y gestionar órdenes.
/// 
/// Almacena:
/// - `usuarios`: Mapeo de `AccountId` a información del `Usuario`.
/// - `productos`: Lista de productos publicados por los vendedores.
/// - `ordenes`: Lista de órdenes creadas por los compradores.
#[ink(storage)]
pub struct MarketplacePrincipal {
    usuarios: Mapping<AccountId, Usuario>,
    productos: Vec<Producto>,
    ordenes: Vec<Orden>,
}

impl MarketplacePrincipal {
    /// Crea una nueva instancia vacía del marketplace.
    #[ink(constructor)]
    pub fn new() -> Self {
        Self {
            usuarios: Mapping::default(),
            productos: Vec::new(),
            ordenes: Vec::new(),
        }
    }

    /// Registra al usuario que invoca el contrato con el rol indicado.
    ///
    /// # Parámetros
    /// - `rol`: Rol que tendrá el usuario (Comprador, Vendedor o Ambos).
    ///
    /// # Errores
    /// Retorna un error si el usuario ya estaba registrado.
    #[ink(message)]
    pub fn registrar_usuario(&mut self, rol: RolUsuario) -> Result<(), String> {
        self.registrar_usuario_interno(rol)
    }

    /// Permite a un vendedor publicar un nuevo producto en el marketplace.
    ///
    /// # Parámetros
    /// - `nombre`: Nombre del producto.
    /// - `descripcion`: Descripción del producto.
    /// - `precio`: Precio por unidad.
    /// - `cantidad`: Cantidad disponible.
    /// - `categoria`: Categoría del producto.
    ///
    /// # Errores
    /// Retorna un error si el usuario no está registrado, no es vendedor,
    /// o la cantidad es inválida.
    #[ink(message)]
    pub fn publicar_producto(
        &mut self,
        nombre: String,
        descripcion: String,
        precio: Balance,
        cantidad: u32,
        categoria: String,
    ) -> Result<(), SistemaError> {
        self.crear_producto_seguro(nombre, descripcion, precio, cantidad, categoria)
    }

    /// Permite a un comprador crear una nueva orden para un producto publicado.
    ///
    /// # Parámetros
    /// - `producto_id`: ID del producto.
    /// - `cantidad`: Cantidad deseada.
    ///
    /// # Retorna
    /// ID de la orden creada.
    ///
    /// # Errores
    /// Retorna un error si el usuario no está registrado, no es comprador,
    /// el producto no existe o no hay suficiente cantidad.
    #[ink(message)]
    pub fn crear_orden(&mut self, producto_id: u32, cantidad: u32) -> Result<u32, SistemaError> {
        self.crear_nueva_orden(producto_id, cantidad)
    }

    /// Permite al vendedor marcar una orden como enviada.
    ///
    /// # Parámetros
    /// - `orden_id`: ID de la orden.
    ///
    /// # Errores
    /// Retorna un error si el estado no puede cambiarse o el usuario no tiene permiso.
    #[ink(message)]
    pub fn marcar_orden_como_enviada(&mut self, orden_id: u32) -> Result<(), SistemaError> {
        self.actualizar_estado_orden(orden_id, EstadoOrden::Enviada)
    }

    /// Permite al comprador marcar una orden como recibida.
    ///
    /// # Parámetros
    /// - `orden_id`: ID de la orden.
    ///
    /// # Errores
    /// Retorna un error si el estado no puede cambiarse o el usuario no tiene permiso.
    #[ink(message)]
    pub fn marcar_como_recibida(&mut self, orden_id: u32) -> Result<(), SistemaError> {
        self.actualizar_estado_orden(orden_id, EstadoOrden::Recibida)
    }

    // --- Funciones internas ---

    /// Registra un nuevo usuario internamente, verificando si ya existe.
    fn registrar_usuario_interno(&mut self, rol: RolUsuario) -> Result<(), SistemaError> {
        let usuario_llamador = self.env().caller();
        self.verificar_registro(usuario_llamador)?;
        let nuevo_usuario = Usuario {
            direccion: usuario_llamador,
            rol,
            reputacion_como_comprador: 0,
            reputacion_como_vendedor: 0,
        };
        self.usuarios.insert(usuario_llamador, &nuevo_usuario);
        Ok(())
    }

    /// Crea un nuevo producto después de verificar los permisos y la cantidad.
    fn crear_producto_seguro(
        &mut self,
        nombre: String,
        descripcion: String,
        precio: Balance,
        cantidad: u32,
        categoria: String,
    ) -> Result<(), SistemaError> {
        let vendedor = self.env().caller();
        self.verificar_registro(vendedor)?;
        self.verificar_rol(vendedor, RolUsuario::Vendedor)?;
        self.verificar_cantidad(cantidad)?;
        self.agregar_producto(nombre, descripcion, precio, cantidad, categoria, vendedor)
    }

    /// Crea una nueva orden de compra para un producto existente.
    fn crear_nueva_orden(&mut self, producto_id: u32, cantidad: u32) -> Result<u32, SistemaError> {
        let comprador = self.env().caller();
        self.verificar_registro(comprador)?;
        self.verificar_rol(comprador, RolUsuario::Comprador)?;

        let producto = self.obtener_producto_mut(producto_id)?;
        if producto.cantidad < cantidad {
            return Err(SistemaError::CantidadInsuficiente);
        }

        producto.cantidad -= cantidad;
        self.crear_y_emitir_orden(comprador, producto.vendedor, producto_id, cantidad)
    }

    /// Actualiza el estado de una orden si el usuario tiene permiso.
    fn actualizar_estado_orden(&mut self, orden_id: u32, nuevo_estado: EstadoOrden) -> Result<(), SistemaError> {
        let caller = self.env().caller();
        self.verificar_registro(caller)?;

        let orden = self.obtener_orden_mut(orden_id)?;
        self.verificar_permiso_orden(caller, orden, nuevo_estado)?;

        let _estado_anterior = orden.estado;
        orden.estado = nuevo_estado;

        self.emitir_evento_estado(orden_id, orden.comprador, nuevo_estado);
        Ok(())
    }

    // --- Funciones auxiliares ---

    /// Verifica que el usuario esté registrado.
    fn verificar_registro(&self, usuario: AccountId) -> Result<(), SistemaError> {
        if !self.usuarios.contains_key(&usuario) {
            Err(SistemaError::UsuarioNoRegistrado)
        } else {
            Ok(())
        }
    }

    /// Verifica que el usuario tenga el rol requerido.
    fn verificar_rol(&self, usuario: AccountId, rol_requerido: RolUsuario) -> Result<(), SistemaError> {
        let usuario_data = self.usuarios.get(&usuario)
            .ok_or(SistemaError::UsuarioNoRegistrado)?;

        match (usuario_data.rol, rol_requerido) {
            (RolUsuario::Ambos, _) => Ok(()),
            (rol, requerido) if rol == requerido => Ok(()),
            _ => Err(SistemaError::NoEsRolCorrecto),
        }
    }

    /// Verifica que la cantidad sea mayor que cero.
    fn verificar_cantidad(&self, cantidad: u32) -> Result<(), SistemaError> {
        if cantidad == 0 {
            Err(SistemaError::CantidadInsuficiente)
        } else {
            Ok(())
        }
    }

    /// Agrega un nuevo producto a la lista del marketplace.
    fn agregar_producto(
        &mut self,
        nombre: String,
        descripcion: String,
        precio: Balance,
        cantidad: u32,
        categoria: String,
        vendedor: AccountId,
    ) -> Result<(), SistemaError> {
        let id = self.productos.len() as u32;
        let nuevo_producto = Producto::new(id, nombre, descripcion, precio, cantidad, categoria, vendedor);
        self.productos.push(nuevo_producto);
        Ok(())
    }

    /// Devuelve una referencia mutable a un producto dado su ID.
    fn obtener_producto_mut(&mut self, id: u32) -> Result<&mut Producto, SistemaError> {
        self.productos
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or(SistemaError::ProductosVacios)
    }

    /// Crea una orden y emite el evento correspondiente.
    fn crear_y_emitir_orden(
        &mut self,
        comprador: AccountId,
        vendedor: AccountId,
        producto_id: u32,
        cantidad: u32
    ) -> Result<u32, SistemaError> {
        let id = self.ordenes.len() as u32;
        let nueva_orden = Orden::new(id, comprador, vendedor, producto_id, cantidad);
        self.ordenes.push(nueva_orden.clone());
        self.emitir_evento_creacion(nueva_orden);
        Ok(id)
    }

    /// Devuelve una referencia mutable a una orden dada su ID.
    fn obtener_orden_mut(&mut self, id: u32) -> Result<&mut Orden, SistemaError> {
        self.ordenes
            .get_mut(id as usize)
            .ok_or(SistemaError::OrdenNoExiste)
    }

    /// Verifica si el usuario tiene permiso para cambiar el estado de una orden.
    fn verificar_permiso_orden(
        &self,
        caller: AccountId,
        orden: &Orden,
        nuevo_estado: EstadoOrden
    ) -> Result<(), SistemaError> {
        match nuevo_estado {
            EstadoOrden::Enviada if caller != orden.vendedor => Err(SistemaError::NoEsRolCorrecto),
            EstadoOrden::Recibida if caller != orden.comprador => Err(SistemaError::NoEsRolCorrecto),
            _ => self.verificar_transicion_estado(orden.estado, nuevo_estado),
        }
    }

    /// Verifica si la transición de estado de la orden es válida.
    fn verificar_transicion_estado(
        &self,
        actual: EstadoOrden,
        nuevo: EstadoOrden
    ) -> Result<(), SistemaError> {
        match (actual, nuevo) {
            (EstadoOrden::Pendiente, EstadoOrden::Enviada) => Ok(()),
            (EstadoOrden::Enviada, EstadoOrden::Recibida) => Ok(()),
            _ => Err(SistemaError::EstadoInvalido),
        }
    }

    // --- Eventos ---

    /// Emite un evento indicando la creación de una nueva orden.
    fn emitir_evento_creacion(&self, orden: Orden) {
        self.env().emit_event(OrdenCreada {
            id: orden.id,
            comprador: orden.comprador,
            vendedor: orden.vendedor,
            producto_id: orden.producto_id,
            cantidad: orden.cantidad,
        });
    }

    /// Emite un evento indicando el cambio de estado de una orden.
    fn emitir_evento_estado(&self, id: u32, comprador: AccountId, estado: EstadoOrden) {
        self.env().emit_event(EstadoOrdenCambiado {
            id,
            comprador,
            nuevo_estado: estado,
        });
    }
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
