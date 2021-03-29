{ pkgs ? import <nixpkgs> {} }:

with pkgs;

mkShell {
  LIBCLANG_PATH="${pkgs.llvmPackages.libclang}/lib";
  PROTOC="${pkgs.protobuf}/bin/protoc";
  PROTOC_INCLUDE="${pkgs.protobuf}/include";
  JAVA_HOME="${pkgs.jdk8}/lib/openjdk";
  shellHook = '' ln -s ${pkgs.jdk8}/lib/openjdk ./query-engine/connector-test-kit/.openjdk '';
  buildInputs = with pkgs; [
    sbt
    sbt-extras
    jdk8
    openssl
    pkg-config
    clangStdenv
    llvmPackages.libclang
    kerberos
    protobuf
  ];
}
