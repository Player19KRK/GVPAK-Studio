// Only these read aliases retain the old brand, so existing browser drafts survive the rename.
export function readLocalSetting(storage,key){
 const current='gvpak-studio-'+key;
 const saved=storage.getItem(current);
 if(saved!==null)return saved;
 const legacy='edge-'+key;
 const value=storage.getItem(legacy);
 if(value!==null){storage.setItem(current,value);storage.removeItem?.(legacy);}
 return value;
}
