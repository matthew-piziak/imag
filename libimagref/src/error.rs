generate_error_module!(
    generate_error_types!(RefError, RefErrorKind,
        StoreReadError          => "Store read error",
        StoreWriteError         => "Store write error",
        IOError                 => "IO Error",
        UTF8Error               => "UTF8 Error",
        HeaderTypeError         => "Header type error",
        HeaderFieldMissingError => "Header field missing error",
        HeaderFieldWriteError   => "Header field cannot be written",
        HeaderFieldReadError    => "Header field cannot be read",
        HeaderFieldAlreadyExistsError => "Header field already exists, cannot override",
        PathUTF8Error => "Path cannot be converted because of UTF8 Error",
        PathHashingError => "Path cannot be hashed",
        PathCanonicalizationError => "Path cannot be canonicalized",

        TypeConversionError => "Couldn't convert types",
        RefToDisplayError => "Cannot convert Ref to string to show it to user",

        RefNotInStore => "Ref/StoreId does not exist in store",

        RefTargetDoesNotExist       => "Ref Target does not exist",
        RefTargetPermissionError    => "Ref Target permissions insufficient for referencing",
        RefTargetCannotBeHashed     => "Ref Target cannot be hashed (is it a directory?)",
        RefTargetFileCannotBeOpened => "Ref Target File cannot be open()ed",
        RefTargetCannotReadPermissions => "Ref Target: Cannot read permissions",

        RefHashingError => "Error while hashing"
    );
);

pub use self::error::RefError;
pub use self::error::RefErrorKind;
pub use self::error::MapErrInto;

