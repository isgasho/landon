precision mediump float;

// TODO: Pass this uniform in
uniform vec3 uCameraPos;

varying vec3 vNormal;
varying vec3 vWorldSpacePos;

struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float specular_intensity;
};

uniform Material material;


// We use a gold material
void main(void) {
  vec3 lightColor = vec3(1.0, 1.0, 1.0);
  vec3 lightPos = vec3(1.0, 5.0, 3.0);

  vec3 ambient = material.ambient;

  vec3 normal = normalize(vNormal);
  vec3 lightDir = normalize(lightPos - vWorldSpacePos);
  float diff = max(dot(normal, lightDir), 0.0);

    //  gold diffuse -> vec3(0.75164, 0.60648, 0.22648);
    // gold specular_intensity -> 32.0

  //  vec3 diffuse = diff * vec3(0.75164, 0.60648, 0.22648);
  vec3 diffuse = diff * material.diffuse;

  float shininess = material.specular_intensity;

  vec3 viewDir = normalize(uCameraPos - vWorldSpacePos);
  vec3 reflectDir = reflect(-lightDir, normal);

  float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32.0);

  vec3 specular = shininess * spec * material.specular;

  vec4 meshColor = vec4(ambient + diffuse + specular, 1.0);

   gl_FragColor = meshColor;
}
