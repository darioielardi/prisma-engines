use crate::common::*;
use datamodel::ast::Span;
use datamodel::diagnostics::DatamodelError;
use datamodel::dml;
use indoc::indoc;

#[test]
fn relation_happy_path() {
    let dml = r#"
    model User {
        id Int @id
        firstName String
        posts Post[]
    }

    model Post {
        id Int @id
        text String
        userId Int
        user User @relation(fields: [userId], references: [id])
    }
    "#;

    let schema = parse(dml);
    let user_model = schema.assert_has_model("User");
    user_model
        .assert_has_relation_field("posts")
        .assert_arity(&dml::FieldArity::List)
        .assert_relation_to("Post")
        .assert_relation_base_fields(&[])
        .assert_relation_referenced_fields(&[]);

    let post_model = schema.assert_has_model("Post");
    post_model
        .assert_has_relation_field("user")
        .assert_arity(&dml::FieldArity::Required)
        .assert_relation_to("User")
        .assert_relation_base_fields(&["userId"])
        .assert_relation_referenced_fields(&["id"]);
}

#[test]
fn relation_must_error_when_base_field_does_not_exist() {
    let dml = r#"
    model User {
        id Int @id
        firstName String
        posts Post[]
    }

    model Post {
        id Int @id
        text String
        user User @relation(fields: [userId], references: [id])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_validation_error("The argument fields must refer only to existing fields. The following fields do not exist in this model: userId", Span::new(154, 210)));
}

#[test]
fn relation_must_error_when_base_field_is_not_scalar() {
    let dml = r#"
    model User {
        id Int @id
        firstName String
        posts Post[]
    }

    model Post {
        id Int @id
        text String
        userId Int
        otherId Int

        user User @relation(fields: [other], references: [id])
        other OtherModel @relation(fields: [otherId], references: [id])
    }

    model OtherModel {
        id Int @id
        posts Post[]
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is_at(0,DatamodelError::new_validation_error("The argument fields must refer only to scalar fields. But it is referencing the following relation fields: other", Span::new(194, 249)));
    errors.assert_is_at(1,DatamodelError::new_attribute_validation_error("The type of the field `other` in the model `Post` is not matching the type of the referenced field `id` in model `User`.", "relation", Span::new(194, 249)));
}

#[test]
fn optional_relation_field_must_succeed_when_all_underlying_fields_are_optional() {
    let dml = r#"
    model User {
        id        Int     @id
        firstName String?
        lastName  String?
        posts     Post[]

        @@unique([firstName, lastName])
    }

    model Post {
        id            Int     @id
        text          String
        userFirstName String?
        userLastName  String?

        user          User?   @relation(fields: [userFirstName, userLastName], references: [firstName, lastName])
    }
    "#;

    // must not crash
    let _ = parse(dml);
}

#[test]
fn required_relation_field_must_error_when_one_underlying_field_is_optional() {
    let dml = r#"
    model User {
        id        Int     @id
        firstName String
        lastName  String?
        posts     Post[]

        @@unique([firstName, lastName])
    }

    model Post {
        id            Int     @id
        text          String
        userFirstName String
        userLastName  String?

        user          User    @relation(fields: [userFirstName, userLastName], references: [firstName, lastName])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_validation_error("The relation field `user` uses the scalar fields userFirstName, userLastName. At least one of those fields is optional. Hence the relation field must be optional as well.", Span::new(320, 426)));
}

#[test]
fn required_relation_field_must_succeed_when_at_least_one_underlying_fields_is_required() {
    let dml = r#"
    model User {
        id        Int     @id
        firstName String
        lastName  String?
        posts     Post[]

        @@unique([firstName, lastName])
    }

    model Post {
        id            Int     @id
        text          String
        userFirstName String
        userLastName  String?

        user          User    @relation(fields: [userFirstName, userLastName], references: [firstName, lastName])
    }
    "#;

    // must not crash
    let _ = parse(dml);
}

#[test]
fn required_relation_field_must_error_when_all_underlying_fields_are_optional() {
    let dml = r#"
    model User {
        id        Int     @id
        firstName String?
        lastName  String?
        posts     Post[]

        @@unique([firstName, lastName])
    }

    model Post {
        id            Int     @id
        text          String
        userFirstName String?
        userLastName  String?

        user          User    @relation(fields: [userFirstName, userLastName], references: [firstName, lastName])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_validation_error("The relation field `user` uses the scalar fields userFirstName, userLastName. All those fields are optional. Hence the relation field must be optional as well.", Span::new(322, 428)));
}

#[test]
fn required_relation_field_must_error_if_it_is_virtual() {
    let dml = r#"
    model User {
        id      Int     @id
        address Address
    }

    model Address {
        id     Int     @id
        userId Int
        user   User    @relation(fields: [userId], references: [id])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_attribute_validation_error(
        "The relation field `address` on Model `User` is required. This is no longer valid because it\'s not possible to enforce this constraint on the database level. Please change the field type from `Address` to `Address?` to fix this.",
        "relation",
        Span::new(54, 70),
    ));
}

#[test]
fn relation_must_error_when_referenced_field_does_not_exist() {
    let dml = r#"
    model User {
        id Int @id
        firstName String
        posts Post[]
    }

    model Post {
        id Int @id
        text String
        userId Int
        user User @relation(fields: [userId], references: [fooBar])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_validation_error("The argument `references` must refer only to existing fields in the related model `User`. The following fields do not exist in the related model: fooBar", Span::new(173, 233)));
}

#[test]
fn relation_must_error_when_referenced_field_is_not_scalar() {
    let dml = r#"
    model User {
        id Int @id
        firstName String
        posts Post[]
    }

    model Post {
        id Int @id
        text String
        userId Int
        user User @relation(fields: [userId], references: [posts])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_validation_error("The argument `references` must refer only to scalar fields in the related model `User`. But it is referencing the following relation fields: posts", Span::new(173, 232)));
}

#[test]
fn relation_must_error_when_referenced_fields_are_not_a_unique_criteria() {
    let dml = r#"
    model User {
        id        Int    @id
        firstName String
        posts     Post[]
    }

    model Post {
        id       Int    @id
        text     String
        userName String
        user     User   @relation(fields: [userName], references: [firstName])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_validation_error("The argument `references` must refer to a unique criteria in the related model `User`. But it is referencing the following fields that are not a unique criteria: firstName", Span::new(205, 276)));
}

#[allow(non_snake_case)]
#[test]
fn relation_must_NOT_error_when_referenced_fields_are_not_a_unique_criteria_on_mysql() {
    // MySQL allows foreign key to references a non unique criteria
    // https://stackoverflow.com/questions/588741/can-a-foreign-key-reference-a-non-unique-index
    let dml = r#"
    datasource db {
        provider = "mysql"
        url = "mysql://localhost:3306"
    }

    model User {
        id        Int    @id
        firstName String
        posts     Post[]
    }

    model Post {
        id       Int    @id
        text     String
        userName String
        user     User   @relation(fields: [userName], references: [firstName])
    }
    "#;

    let _ = parse(dml);
}

#[test]
fn relation_must_error_when_referenced_fields_are_multiple_uniques() {
    let dml = r#"
    model User {
        id Int @id
        firstName String @unique
        posts Post[]
    }

    model Post {
        id       Int    @id
        text     String
        userId   Int
        userName String
        // the relation is referencing two uniques. That is too much.
        user User @relation(fields: [userId, userName], references: [id, firstName])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_validation_error("The argument `references` must refer to a unique criteria in the related model `User`. But it is referencing the following fields that are not a unique criteria: id, firstName", Span::new(290, 367)));
}

#[test]
fn relation_must_error_when_types_of_base_field_and_referenced_field_do_not_match() {
    let dml = r#"
    model User {
        id        Int @id
        firstName String
        posts     Post[]
    }

    model Post {
        id     Int     @id
        userId String  // this type does not match
        user   User    @relation(fields: [userId], references: [id])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_attribute_validation_error("The type of the field `userId` in the model `Post` is not matching the type of the referenced field `id` in model `User`.", "relation", Span::new(204, 265)));
}

#[test]
fn relation_must_error_when_number_of_fields_and_references_is_not_equal() {
    let dml = r#"
    model User {
        id        Int @id
        firstName String
        posts     Post[]
    }

    model Post {
        id       Int     @id
        userId   Int
        userName String
        user     User    @relation(fields: [userId, userName], references: [id])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_attribute_validation_error(
        "You must specify the same number of fields in `fields` and `references`.",
        "relation",
        Span::new(200, 273),
    ));
}

#[test]
fn relation_must_succeed_when_type_alias_is_used_for_referenced_field() {
    let dml = r#"
    type CustomId = Int @id @default(autoincrement())

    model User {
        id        CustomId
        firstName String
        posts     Post[]
    }

    model Post {
        id     Int     @id
        userId Int
        user   User    @relation(fields: [userId], references: [id])
    }
    "#;

    let _ = parse(dml);
}

#[test]
fn must_error_when_references_argument_is_missing_for_one_to_many() {
    let dml = r#"
    model User {
        id        Int @id
        firstName String
        posts     Post[]
    }

    model Post {
        id     Int     @id
        userId Int
        user   User    @relation(fields: [userId])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_attribute_validation_error(
        "The relation field `user` on Model `Post` must specify the `references` argument in the @relation attribute.",
        "relation",
        Span::new(172, 215),
    ));
}

#[test]
fn must_error_fields_or_references_argument_is_placed_on_wrong_side_for_one_to_many() {
    let dml = r#"
        datasource pg {
            provider = "postgres"
            url = "postgresql://localhost:5432"
        }

        model User {
          id     Int    @id
          postId Int[]
          posts  Post[] @relation(fields: [postId], references: [id])
        }

        model Post {
          id     Int   @id
          userId Int?
          user   User? @relation(fields: [userId], references: [id])
        }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(
        DatamodelError::new_attribute_validation_error(
            "The relation field `posts` on Model `User` must not specify the `fields` or `references` argument in the @relation attribute. You must only specify it on the opposite field `user` on model `Post`.",
            "relation", Span::new(200, 260)
        ),
    );
}

#[test]
fn must_error_when_both_arguments_are_missing_for_one_to_many() {
    let dml = r#"
    model User {
        id        Int @id
        firstName String
        posts     Post[]
    }

    model Post {
        id     Int     @id
        userId Int
        user   User
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is_at(
        0,
        DatamodelError::new_attribute_validation_error(
            "The relation field `user` on Model `Post` must specify the `fields` argument in the @relation attribute. You can run `prisma format` to fix this automatically.",
            "relation",
            Span::new(172, 184),
        ),
    );
    errors.assert_is_at(1, DatamodelError::new_attribute_validation_error(
        "The relation field `user` on Model `Post` must specify the `references` argument in the @relation attribute.",
        "relation",
        Span::new(172, 184),
    ));
}

#[test]
fn must_error_when_fields_argument_is_missing_for_one_to_one() {
    let dml = r#"
    model User {
        id        Int @id
        firstName String
        post      Post?
    }

    model Post {
        id     Int     @id
        userId Int
        user   User    @relation(references: [id])
    }
    "#;

    let errors = parse_error(dml);

    errors.assert_are(
        &[DatamodelError::new_attribute_validation_error("The relation fields `post` on Model `User` and `user` on Model `Post` do not provide the `fields` argument in the @relation attribute. You have to provide it on one of the two fields.", "relation",Span::new(77, 93)), 
            DatamodelError::new_attribute_validation_error("The relation fields `user` on Model `Post` and `post` on Model `User` do not provide the `fields` argument in the @relation attribute. You have to provide it on one of the two fields.", "relation",Span::new(171, 214))],
    );
}

#[test]
fn must_error_when_references_argument_is_missing_for_one_to_one() {
    let dml = r#"
    model User {
        id        Int @id
        firstName String
        post      Post
    }

    model Post {
        id     Int     @id
        userId Int
        user   User    @relation(fields: [userId])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is_at(
        0,
        DatamodelError::new_attribute_validation_error(
            "The relation fields `post` on Model `User` and `user` on Model `Post` do not provide the `references` argument in the @relation attribute. You have to provide it on one of the two fields.",
            "relation", Span::new(77, 92)
        ),
    );
    errors.assert_is_at(
        1,
        DatamodelError::new_attribute_validation_error(
            "The relation fields `user` on Model `Post` and `post` on Model `User` do not provide the `references` argument in the @relation attribute. You have to provide it on one of the two fields.",
            "relation", Span::new(170, 213)
        ),
    );
}

#[test]
fn must_error_when_fields_and_references_argument_are_placed_on_different_sides_for_one_to_one() {
    let dml = r#"
    model User {
        id        Int @id
        firstName String
        postId    Int
        post      Post @relation(references: [id])
    }

    model Post {
        id     Int     @id
        userId Int
        user   User    @relation(fields: [userId])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is_at(
        0,
        DatamodelError::new_attribute_validation_error(
            "The relation field `post` on Model `User` provides the `references` argument in the @relation attribute. And the related field `user` on Model `Post` provides the `fields` argument. You must provide both arguments on the same side.",
            "relation", Span::new(99, 142)
        ),
    );
    errors.assert_is_at(
        1,
        DatamodelError::new_attribute_validation_error(
            "The relation field `user` on Model `Post` provides the `fields` argument in the @relation attribute. And the related field `post` on Model `User` provides the `references` argument. You must provide both arguments on the same side.",
            "relation", Span::new(220, 263)
        ),
    );
}

#[test]
fn must_error_when_fields_or_references_argument_is_placed_on_both_sides_for_one_to_one() {
    let dml = r#"
    model User {
        id        Int @id
        firstName String
        postId    Int
        post      Post @relation(fields: [postId], references: [id])
    }

    model Post {
        id     Int     @id
        userId Int
        user   User    @relation(fields: [userId], references: [id])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is_at(
            0,
            DatamodelError::new_attribute_validation_error(
                "The relation fields `post` on Model `User` and `user` on Model `Post` both provide the `references` argument in the @relation attribute. You have to provide it only on one of the two fields.",
                "relation", Span::new(99, 160)
            ),
        );
    errors.assert_is_at(
            1,
            DatamodelError::new_attribute_validation_error(
                "The relation fields `post` on Model `User` and `user` on Model `Post` both provide the `fields` argument in the @relation attribute. You have to provide it only on one of the two fields.",
                "relation", Span::new(99, 160)
            ),
        );

    errors.assert_is_at(
        2,
        DatamodelError::new_attribute_validation_error(
            "The relation fields `user` on Model `Post` and `post` on Model `User` both provide the `references` argument in the @relation attribute. You have to provide it only on one of the two fields.",
            "relation", Span::new(238, 299)
        ),
    );

    errors.assert_is_at(
        3,
        DatamodelError::new_attribute_validation_error(
            "The relation fields `user` on Model `Post` and `post` on Model `User` both provide the `fields` argument in the @relation attribute. You have to provide it only on one of the two fields.",
            "relation", Span::new(238,299)
        ),
    );
}

#[test]
fn must_error_for_required_one_to_one_self_relations() {
    let dml = r#"
        model User {
          id       Int  @id
          friendId Int
          friend   User @relation("Friends", fields: friendId, references: id)
          friendOf User @relation("Friends")
        }
    "#;
    let errors = parse_error(dml);
    errors.assert_is(

        DatamodelError::new_attribute_validation_error("The relation field `friendOf` on Model `User` is required. This is no longer valid because it\'s not possible to enforce this constraint on the database level. Please change the field type from `User` to `User?` to fix this.", "relation",  Span::new(162, 197)),
    );
}

#[test]
fn must_error_when_non_id_field_is_referenced_in_a_many_to_many() {
    let dml = r#"
    model Post {
      id         Int        @id
      slug       Int        @unique
      categories Category[] @relation(references: [id])
   }

   model Category {
     id    Int    @id @default(autoincrement())
     posts Post[] @relation(references: [slug])
   }"#;

    let errors = parse_error(dml);
    errors.assert_is_at(
        0,
        DatamodelError::new_validation_error(
            "Many to many relations must always reference the id field of the related model. Change the argument `references` to use the id field of the related model `Post`. But it is referencing the following fields that are not the id: slug",
            Span::new(221, 264)
        ),
    );
}

#[test]
fn must_error_nicely_when_a_many_to_many_is_not_possible() {
    // many 2 many is not possible because Post does not have a singular id field
    let dml = r#"
    model Post {
      id         Int
      slug       Int        @unique
      categories Category[] @relation(references: [id])

      @@id([id, slug])
   }

   model Category {
     id    Int    @id @default(autoincrement())
     posts Post[] @relation(references: [slug])
   }"#;

    let errors = parse_error(dml);
    errors.assert_is_at(0, DatamodelError::new_field_validation_error(
        "The relation field `posts` on Model `Category` references `Post` which does not have an `@id` field. Models without `@id` can not be part of a many to many relation. Use an explicit intermediate Model to represent this relationship.",
        "Category",
        "posts",
        Span::new(234, 277)
    ));
}

#[test]
fn must_error_when_many_to_many_is_not_possible_due_to_missing_id() {
    let dml = r#"
    // Post does not have @id
    model Post {
     slug       Int        @unique
     categories Category[]
   }

   model Category {
     id    Int    @id @default(autoincrement())
     posts Post[]
   }"#;

    let errors = parse_error(dml);
    errors.assert_is_at(
        0,
        DatamodelError::new_field_validation_error(
            "The relation field `posts` on Model `Category` references `Post` which does not have an `@id` field. Models without `@id` can not be part of a many to many relation. Use an explicit intermediate Model to represent this relationship.",
            "Category",
            "posts",
            Span::new(189, 202)
        ),
    );
}

#[test]
fn must_allow_relations_with_default_native_types_with_annotation_on_one_side() {
    let dm1 = indoc! {
        r#"
        datasource db {
            provider = "mysql"
            url      = "mysql://"
        }

        model Blog {
            id        Int   @id
            authorId  Int @db.Int
            author    User @relation(fields: [authorId], references: [id])
        }

        model User {
            id        Int @id
            blogs     Blog[]
        }
        "#
    };

    let dm2 = indoc! {
        r#"
        datasource db {
            provider = "mysql"
            url      = "mysql://"
        }

        model Blog {
            id        Int   @id
            authorId  Int
            author    User @relation(fields: [authorId], references: [id])
        }

        model User {
            id        Int @id @db.Int
            blogs     Blog[]

        }
        "#
    };

    let dm3 = indoc! {
        r#"
        datasource db {
            provider = "mysql"
            url      = "mysql://"
        }

        model Blog {
            id        Int   @id
            authorId  Int?   @db.Int
            author    User?  @relation(fields: [authorId], references: [id])
        }

        model User {
            id        Int @id @db.Int
            blogs     Blog[]
        }
        "#
    };

    for dm in &[dm1, dm2, dm3] {
        assert!(
            datamodel::parse_datamodel(dm).is_ok(),
            "{:?}",
            datamodel::parse_datamodel(dm).unwrap_err()
        );
    }
}
