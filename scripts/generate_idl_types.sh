

generate_idl_type(){
    FILE=$1
    IDL_DIR=$2
    echo $FILE
    FILE_NAME="$(basename $FILE .json)"
    PREFIX=$(tr '[:lower:]' '[:upper:]' <<< ${FILE_NAME:0:1})${FILE_NAME:1}
    echo $FILE_NAME_U
    OUT_PATH="$IDL_DIR/$FILE_NAME.ts"

    # output types 
    type_name="${PREFIX}IDL"
    echo "export type $type_name = " >> $OUT_PATH
    cat $FILE >> $OUT_PATH 
    echo ";" >> $OUT_PATH

    # output json
    json_name="${PREFIX}JSON"
    echo "export const $json_name: $type_name =" >> $OUT_PATH
    cat $FILE >> $OUT_PATH

}

# IDL output directory
IDL_DIR="packages/idls"
rm -rf $IDL_DIR
mkdir -p $IDL_DIR

IDL_JSON='temp/idl/*.json'
if [ $(ls -l temp/idl/ | wc -l) -ne 0 ]; then 
    for json_file in $IDL_JSON ; do 
        # generate type and IDL
        generate_idl_type $json_file $IDL_DIR
    done
    yarn prettier --write $IDL_DIR
else 
    echo "Could not find any IDLs in temp/idl/"
fi