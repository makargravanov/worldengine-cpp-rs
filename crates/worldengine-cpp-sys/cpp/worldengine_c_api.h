#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct we_world we_world;

typedef struct we_world_params
{
   const char* name;
   uint32_t    width;
   uint32_t    height;
   uint32_t    seed;
   uint32_t    num_plates;
   float       ocean_level;
   int32_t     step;
   bool        fade_borders;
} we_world_params;

enum
{
   WE_STEP_PLATES         = 0,
   WE_STEP_PRECIPITATIONS = 1,
   WE_STEP_FULL           = 2
};

we_world* we_world_generate(const we_world_params* params);
void      we_world_free(we_world* world);

uint32_t we_world_width(const we_world* world);
uint32_t we_world_height(const we_world* world);
uint32_t we_world_seed(const we_world* world);
size_t   we_world_len(const we_world* world);

bool we_world_has_biome(const we_world* world);
bool we_world_has_humidity(const we_world* world);
bool we_world_has_icecap(const we_world* world);
bool we_world_has_irrigation(const we_world* world);
bool we_world_has_lakemap(const we_world* world);
bool we_world_has_permeability(const we_world* world);
bool we_world_has_precipitation(const we_world* world);
bool we_world_has_rivermap(const we_world* world);
bool we_world_has_temperature(const we_world* world);
bool we_world_has_watermap(const we_world* world);

const float*    we_world_elevation(const we_world* world);
const uint8_t*  we_world_ocean(we_world* world);
const uint16_t* we_world_plates(const we_world* world);
const uint32_t* we_world_biome(we_world* world);
const float*    we_world_humidity(const we_world* world);
const float*    we_world_icecap(const we_world* world);
const float*    we_world_irrigation(const we_world* world);
const float*    we_world_lakemap(const we_world* world);
const float*    we_world_permeability(const we_world* world);
const float*    we_world_precipitation(const we_world* world);
const float*    we_world_rivermap(const we_world* world);
const float*    we_world_sea_depth(const we_world* world);
const float*    we_world_temperature(const we_world* world);
const float*    we_world_watermap(const we_world* world);

#ifdef __cplusplus
}
#endif

