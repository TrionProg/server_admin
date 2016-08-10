use std::error::Error;

pub struct Version {
    versionHash:u32,
}

impl Version {
    pub fn parse( string:&String ) -> Result< Version, String >{
        let mut versionHash:u32=0;
        let mut c=0;

        for ns in string.split('.'){
            match ns.parse::<u32>(){
                Ok( n ) => {
                    if n>255 {
                        return Err( format!("Max value of part of version must be less then 256"));
                    }

                    c+=1;
                    if c>4 {
                        return Err( format!("Version is too long, version should have 4 parts like *.*.*.*"));
                    }

                    versionHash*=256;
                    versionHash+=n;
                },
                Err( e )=>return Err( format!("Can not parse version: {}", e.description())),
            }
        }

        if c!=4 {
            return Err( format!("Version is too short, version should have 4 parts like *.*.*.*"));
        }

        Ok(Version{
            versionHash:versionHash,
        })
    }

    pub fn isNewer( &self, other:&Version ) -> bool{
        self.versionHash>other.versionHash
    }
}
