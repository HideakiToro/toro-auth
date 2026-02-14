# Toro-Auth
An actix-web extension adding basic authentication services.

# IMPORTANT
There is no form of encryption. If you want this to be secure, please start a pull request.

## Includes
1. Core - This crate is the glue of the project and the one that actually registers the routes.
2. Mongo Backend - This is a basic implementation of a backend connecting to mongodb.
3. Example - An example showcasing how easy this project makes adding authentication services to your WebApp.

## How-To
To use this project, your user-/identity-struct should have the following properties:
- username
- password
- id